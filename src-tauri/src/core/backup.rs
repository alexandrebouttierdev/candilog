//! Backup et validation des bases SQLite Candilog.

use crate::core::config::restreindre_fichier;
use crate::core::database::{
    open_pool, run_local_migrations, validate_current_schema, validate_database_file, SqlitePool,
};
use crate::core::errors::{AppError, AppResult};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Exporte un instantané cohérent via l'API backup SQLite.
///
/// Le fichier produit est restreint à son propriétaire avant d'être rempli : il contient
/// l'intégralité des données personnelles, et le `umask` de session le laisserait autrement
/// en `644` — y compris la copie de secours de [`import`], qui survit à un échec de
/// restauration (`docs/DATA.md`).
///
/// # Errors
/// Retourne une erreur si la source ou la destination ne peuvent pas être ouvertes.
pub fn export(pool: &SqlitePool, destination: &Path) -> AppResult<()> {
    let source = pool
        .get()
        .map_err(|error| AppError::Database(error.to_string()))?;
    source.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;
    let mut target = rusqlite::Connection::open(destination)?;
    // Avant l'écriture : entre la création du fichier et la fin de la copie, il ne doit
    // exister aucune fenêtre pendant laquelle les données seraient lisibles par un tiers.
    restreindre_fichier(destination);
    let backup = rusqlite::backup::Backup::new(&source, &mut target)?;
    backup
        .run_to_completion(5, std::time::Duration::from_millis(100), None)
        .map_err(AppError::from)
}

/// Vérifie qu'un fichier est une base SQLite Candilog intègre et compatible.
///
/// # Errors
/// Retourne une erreur si l'en-tête, l'intégrité ou les tables indispensables sont invalides.
pub fn validate(path: &Path) -> AppResult<()> {
    let mut entete = [0_u8; 16];
    let mut file = std::fs::File::open(path)
        .map_err(|error| AppError::Database(format!("Impossible de lire le backup : {error}")))?;
    let trop_court = matches!(
        file.read_exact(&mut entete),
        Err(ref error) if error.kind() == std::io::ErrorKind::UnexpectedEof
    );
    if trop_court || &entete != b"SQLite format 3\0" {
        return Err(AppError::Validation(
            "Le fichier sélectionné n'est pas une base SQLite.".into(),
        ));
    }
    validate_database_file(path)?;
    let connection =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(AppError::Validation(format!(
            "Le backup SQLite est corrompu : {integrity}"
        )));
    }
    for table in [
        "applications",
        "companies",
        "contacts",
        "settings",
        "profile",
        "sectors",
        "professional_domains",
        "company_types",
        "contract_types",
        "status_history",
        "follow_ups",
        "interviews",
        "resume_versions",
        "cover_letters",
    ] {
        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [table],
            |row| row.get(0),
        )?;
        if count != 1 {
            return Err(AppError::Validation(format!(
                "Le backup ne contient pas la table Candilog {table}."
            )));
        }
    }
    Ok(())
}

/// Remplace le contenu de la base active par un backup validé, avec retour arrière.
///
/// # Errors
/// Retourne une erreur si le backup est invalide, ou si la restauration échoue — auquel cas
/// la base active a été remise dans son état antérieur.
pub fn import(pool: &SqlitePool, db_path: &Path, source_path: &Path) -> AppResult<()> {
    validate(source_path)?;
    let candidate = prepare_candidate(db_path, source_path)?;
    let mut rollback = TemporaryDatabase::new(unique_database_path(db_path, "candilog-secours"));
    export(pool, rollback.path())?;
    tracing::info!("copie de secours prise avant restauration");

    let replacement = replace(pool, candidate.path()).and_then(|()| validate_current_schema(pool));
    let Err(failure) = replacement else {
        tracing::info!("backup restauré");
        return Ok(());
    };
    tracing::error!(error = %failure, "restauration échouée, retour arrière");

    let rollback_result =
        replace(pool, rollback.path()).and_then(|()| validate_current_schema(pool));
    rollback.preserve();
    match rollback_result {
        Ok(()) => Err(AppError::Validation(format!(
            "Le backup n'a pas pu être restauré. La base d'origine a été restaurée : vos \
             données sont intactes. Une copie de secours est conservée dans {}.",
            rollback.path().display()
        ))),
        Err(loss) => {
            tracing::error!(error = %loss, "retour arrière échoué");
            Err(AppError::Database(format!(
                "Le backup n'a pas pu être restauré et la base d'origine n'a pas pu être \
                 remise en place. Une copie intacte de vos données est conservée dans {}.",
                rollback.path().display()
            )))
        }
    }
}

fn prepare_candidate(db_path: &Path, source_path: &Path) -> AppResult<TemporaryDatabase> {
    let candidate = TemporaryDatabase::new(unique_database_path(db_path, "candilog-restauration"));
    copy_database(source_path, candidate.path())?;
    let candidate_pool = open_pool(Some(candidate.path()))?;
    run_local_migrations(&candidate_pool)?;
    validate_current_schema(&candidate_pool)?;
    drop(candidate_pool);
    Ok(candidate)
}

fn copy_database(source_path: &Path, destination_path: &Path) -> AppResult<()> {
    let source = rusqlite::Connection::open_with_flags(
        source_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    let mut target = rusqlite::Connection::open(destination_path)?;
    restreindre_fichier(destination_path);
    let backup = rusqlite::backup::Backup::new(&source, &mut target)?;
    backup
        .run_to_completion(5, std::time::Duration::from_millis(100), None)
        .map_err(AppError::from)
}

fn replace(pool: &SqlitePool, source_path: &Path) -> AppResult<()> {
    {
        let source = rusqlite::Connection::open_with_flags(
            source_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        let mut target = pool
            .get()
            .map_err(|error| AppError::Database(error.to_string()))?;
        let backup = rusqlite::backup::Backup::new(&source, &mut target)?;
        backup
            .run_to_completion(5, std::time::Duration::from_millis(100), None)
            .map_err(AppError::from)?;
    }
    run_local_migrations(pool)
}

fn unique_database_path(db_path: &Path, prefix: &str) -> PathBuf {
    let directory = db_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir);
    directory.join(format!("{prefix}-{}.sqlite", uuid::Uuid::new_v4()))
}

struct TemporaryDatabase {
    path: PathBuf,
    delete_on_drop: bool,
}

impl TemporaryDatabase {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            delete_on_drop: true,
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn preserve(&mut self) {
        self.delete_on_drop = false;
    }
}

impl Drop for TemporaryDatabase {
    fn drop(&mut self) {
        if !self.delete_on_drop {
            return;
        }
        for path in [
            self.path.clone(),
            sidecar_path(&self.path, "-wal"),
            sidecar_path(&self.path, "-shm"),
        ] {
            if let Err(error) = std::fs::remove_file(&path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!(path = %path.display(), %error, "fichier temporaire non supprimé");
                }
            }
        }
    }
}

fn sidecar_path(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

/// Vide les données utilisateur en conservant le schéma, les migrations et les quatre
/// référentiels métier.
///
/// Les référentiels ne sont pas des données utilisateur : les effacer laisserait les
/// formulaires sans options et rendrait toute nouvelle candidature impossible à créer, la
/// clé étrangère du contrat étant `NOT NULL`.
///
/// # Errors
/// Retourne une erreur si la transaction SQLite échoue.
pub fn reset_data(pool: &SqlitePool) -> AppResult<()> {
    let mut connection = pool
        .get()
        .map_err(|error| AppError::Database(error.to_string()))?;
    let transaction = connection.transaction()?;
    transaction.execute_batch(
        "DELETE FROM follow_ups;
         DELETE FROM interviews;
         DELETE FROM status_history;
         DELETE FROM applications;
         DELETE FROM contacts;
         DELETE FROM companies;
         DELETE FROM resume_versions;
         DELETE FROM cover_letters;
         DELETE FROM profile;
         DELETE FROM settings;
         DELETE FROM app_kv;",
    )?;
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::{open_pool, run_local_migrations, LATEST_SCHEMA_VERSION};

    #[test]
    fn validation_refuse_un_fichier_texte() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("faux.sqlite");
        std::fs::write(&path, b"pas une base").unwrap();
        assert!(validate(&path).is_err());
    }

    #[test]
    fn backup_historique_est_refuse_sans_modifier_la_source() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("historique.sqlite");
        {
            let connection = rusqlite::Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE candidatures(id TEXT PRIMARY KEY);
                     INSERT INTO candidatures(id) VALUES ('historique');
                     PRAGMA user_version = 9;",
                )
                .unwrap();
        }
        let before = std::fs::read(&path).unwrap();

        let error = validate(&path).unwrap_err();

        assert!(matches!(error, AppError::IncompatibleData(_)));
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn export_produit_une_base_candilog_valide() {
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("copie.sqlite");
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        export(&pool, &destination).unwrap();
        validate(&destination).unwrap();
    }

    /// La copie de secours prise par [`import`] et la sauvegarde choisie par l'utilisateur
    /// portent l'intégralité des données personnelles. Créées sous le `umask` de session,
    /// elles étaient lisibles en `644` alors que la base elle-même est en `600`, et la copie
    /// de secours survit à un échec de restauration.
    #[cfg(unix)]
    #[test]
    fn une_sauvegarde_n_est_lisible_que_par_son_proprietaire() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("copie.sqlite");
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();

        export(&pool, &destination).unwrap();

        let mode = std::fs::metadata(&destination)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(
            mode & 0o777,
            0o600,
            "mode {:o} au lieu de 600",
            mode & 0o777
        );
    }

    #[test]
    fn reset_conserve_les_referentiels_metier() {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        {
            let conn = pool.get().unwrap();
            conn.execute(
                "INSERT INTO app_kv (kv_key, kv_value) VALUES ('marque', 'oui')",
                [],
            )
            .unwrap();
        }
        reset_data(&pool).unwrap();
        let conn = pool.get().unwrap();
        let kv: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_kv", [], |row| row.get(0))
            .unwrap();
        assert_eq!(kv, 0);
        // Les quatre catalogues survivent à la remise à zéro : sans eux, plus aucune
        // candidature ne pourrait être créée.
        for (table, attendu) in [
            ("sectors", 23),
            ("professional_domains", 22),
            ("contract_types", 22),
            ("company_types", 38),
        ] {
            let total: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(total, attendu, "{table} vidé par la remise à zéro");
        }
    }

    #[test]
    fn import_echoue_sans_detruire_la_base_active() {
        let directory = tempfile::tempdir().unwrap();
        let db_path = directory.path().join("candilog.sqlite");
        let source_path = directory.path().join("etranger.sqlite");

        let pool = open_pool(Some(&db_path)).unwrap();
        run_local_migrations(&pool).unwrap();
        {
            let connection = pool.get().unwrap();
            connection
                .execute_batch(
                    "INSERT INTO companies (id, name, created_at, updated_at)
                        VALUES ('e1', 'Acme', '2026-01-01', '2026-01-01');
                     INSERT INTO applications
                        (id, company_id, job_title, contract_type_code, status, sent_date, created_at, updated_at)
                        VALUES ('c1', 'e1', 'Dev', 'CDI', 'EN_ATTENTE', '2026-01-01', '2026-01-01', '2026-01-01');",
                )
                .unwrap();
        }

        {
            let connection = rusqlite::Connection::open(&source_path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE applications (id TEXT PRIMARY KEY);
                     CREATE TABLE companies (id TEXT PRIMARY KEY);
                     CREATE TABLE contacts (id TEXT PRIMARY KEY);
                     CREATE TABLE settings (id INTEGER PRIMARY KEY);
                     CREATE TABLE profile (id INTEGER PRIMARY KEY);",
                )
                .unwrap();
        }

        let error = import(&pool, &db_path, &source_path).unwrap_err();
        assert!(
            matches!(&error, AppError::IncompatibleData(_)),
            "un backup incomplet doit être refusé sans toucher à la base active : {error}"
        );

        let connection = pool.get().unwrap();
        let job_title: String = connection
            .query_row(
                "SELECT job_title FROM applications WHERE id = 'c1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(job_title, "Dev");
    }

    #[test]
    fn import_refuse_un_schema_courant_incomplet_sans_toucher_la_base_active() {
        let directory = tempfile::tempdir().unwrap();
        let db_path = directory.path().join("candilog.sqlite");
        let source_path = directory.path().join("schema-incomplet.sqlite");

        let active_pool = open_pool(Some(&db_path)).unwrap();
        run_local_migrations(&active_pool).unwrap();
        {
            let active_connection = active_pool.get().unwrap();
            active_connection
                .execute(
                    "INSERT INTO companies (id, name, created_at, updated_at)
                     VALUES ('sentinelle', 'Entreprise intacte', '2026-01-01', '2026-01-01')",
                    [],
                )
                .unwrap();
        }

        {
            let source_connection = rusqlite::Connection::open(&source_path).unwrap();
            source_connection
                .execute_batch(
                    "CREATE TABLE applications (id TEXT PRIMARY KEY);
                     CREATE TABLE companies (id TEXT PRIMARY KEY);
                     CREATE TABLE contacts (id TEXT PRIMARY KEY);
                     CREATE TABLE settings (id INTEGER PRIMARY KEY);
                     CREATE TABLE profile (id INTEGER PRIMARY KEY);
                     CREATE TABLE sectors (id TEXT PRIMARY KEY);
                     CREATE TABLE professional_domains (id TEXT PRIMARY KEY);
                     CREATE TABLE company_types (id TEXT PRIMARY KEY);
                     CREATE TABLE contract_types (id TEXT PRIMARY KEY);
                     CREATE TABLE status_history (id TEXT PRIMARY KEY);
                     CREATE TABLE follow_ups (id TEXT PRIMARY KEY);
                     CREATE TABLE interviews (id TEXT PRIMARY KEY);
                     CREATE TABLE resume_versions (id TEXT PRIMARY KEY);
                     CREATE TABLE cover_letters (id TEXT PRIMARY KEY);
                     PRAGMA user_version = 2;",
                )
                .unwrap();
        }

        let error = import(&active_pool, &db_path, &source_path).unwrap_err();

        assert!(
            matches!(error, AppError::Validation(_) | AppError::Database(_)),
            "un schéma incomplet doit être refusé : {error}"
        );
        let active_connection = active_pool.get().unwrap();
        let company_name: String = active_connection
            .query_row(
                "SELECT name FROM companies WHERE id = 'sentinelle'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(company_name, "Entreprise intacte");
    }

    #[test]
    fn import_restaure_une_sauvegarde_valide() {
        let directory = tempfile::tempdir().unwrap();
        let db_path = directory.path().join("candilog.sqlite");
        let source_path = directory.path().join("sauvegarde.sqlite");
        let source_pool = open_pool(None).unwrap();
        run_local_migrations(&source_pool).unwrap();
        source_pool
            .get()
            .unwrap()
            .execute(
                "INSERT INTO companies (id, name, created_at, updated_at)
                 VALUES ('source', 'Entreprise sauvegardée', '2026-01-01', '2026-01-01')",
                [],
            )
            .unwrap();
        export(&source_pool, &source_path).unwrap();

        let active_pool = open_pool(Some(&db_path)).unwrap();
        run_local_migrations(&active_pool).unwrap();
        active_pool
            .get()
            .unwrap()
            .execute(
                "INSERT INTO companies (id, name, created_at, updated_at)
                 VALUES ('active', 'Entreprise remplacée', '2026-01-01', '2026-01-01')",
                [],
            )
            .unwrap();

        import(&active_pool, &db_path, &source_path).unwrap();

        let active_connection = active_pool.get().unwrap();
        let company_name: String = active_connection
            .query_row(
                "SELECT name FROM companies WHERE id = 'source'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(company_name, "Entreprise sauvegardée");
        let replaced: i64 = active_connection
            .query_row(
                "SELECT count(*) FROM companies WHERE id = 'active'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(replaced, 0);
    }

    #[test]
    fn import_migre_la_copie_candidate_avant_le_remplacement() {
        let directory = tempfile::tempdir().unwrap();
        let db_path = directory.path().join("candilog.sqlite");
        let source_path = directory.path().join("sauvegarde-v1.sqlite");
        {
            let source_connection = rusqlite::Connection::open(&source_path).unwrap();
            source_connection
                .execute_batch(include_str!("../../migrations/init_schema.sql"))
                .unwrap();
            source_connection
                .pragma_update(None, "user_version", 1)
                .unwrap();
        }
        let active_pool = open_pool(Some(&db_path)).unwrap();
        run_local_migrations(&active_pool).unwrap();

        import(&active_pool, &db_path, &source_path).unwrap();

        let active_connection = active_pool.get().unwrap();
        let version: i64 = active_connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        let added_columns: i64 = active_connection
            .query_row(
                "SELECT count(*) FROM pragma_table_info('cover_letters')
                 WHERE name IN ('recipient', 'recipient_address', 'job_reference')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, LATEST_SCHEMA_VERSION);
        assert_eq!(added_columns, 3);
    }
}
