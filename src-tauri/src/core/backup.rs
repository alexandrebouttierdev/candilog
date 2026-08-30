//! Backup et validation des bases SQLite Candilog.

use crate::core::database::{run_local_migrations, validate_database_file, SqlitePool};
use crate::core::errors::{AppError, AppResult};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Exporte un instantané cohérent via l'API backup SQLite.
///
/// # Errors
/// Retourne une erreur si la source ou la destination ne peuvent pas être ouvertes.
pub fn export(pool: &SqlitePool, destination: &Path) -> AppResult<()> {
    let source = pool
        .get()
        .map_err(|error| AppError::Database(error.to_string()))?;
    source.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;
    let mut target = rusqlite::Connection::open(destination)?;
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
    let secours = path_de_secours(db_path);
    export(pool, &secours)?;
    tracing::info!("copie de secours prise avant restauration");

    let Err(echec) = remplacer(pool, source_path) else {
        tracing::info!("backup restauré");
        if secours.exists() {
            if let Err(error) = std::fs::remove_file(&secours) {
                tracing::warn!(path = %secours.display(), %error, "copie de secours non supprimée");
            }
        }
        return Ok(());
    };
    tracing::error!(error = %echec, "restauration échouée, retour arrière");

    match remplacer(pool, &secours) {
        Ok(()) => Err(AppError::Validation(
            "Le backup n'a pas pu être restauré. La base d'origine a été restaurée : vos \
             données sont intactes."
                .into(),
        )),
        Err(perte) => {
            tracing::error!(error = %perte, "retour arrière échoué");
            Err(AppError::Database(format!(
                "Le backup n'a pas pu être restauré et la base d'origine n'a pas pu être \
                 remise en place. Une copie intacte de vos données est conservée dans {}.",
                secours.display()
            )))
        }
    }
}

fn remplacer(pool: &SqlitePool, source_path: &Path) -> AppResult<()> {
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

fn path_de_secours(db_path: &Path) -> PathBuf {
    if db_path.as_os_str().is_empty() {
        std::env::temp_dir().join(format!("candilog-secours-{}.sqlite", uuid::Uuid::new_v4()))
    } else {
        db_path.with_extension("sqlite.bak")
    }
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
         DELETE FROM llm_calls;
         DELETE FROM ats_scores;
         DELETE FROM ai_cache;
         DELETE FROM app_kv;",
    )?;
    transaction.commit()?;
    Ok(())
}

/// Vide uniquement le cache des réponses IA.
///
/// # Errors
/// Retourne une erreur si la table ne peut pas être vidée.
pub fn clear_ai_cache(pool: &SqlitePool) -> AppResult<()> {
    let connection = pool
        .get()
        .map_err(|error| AppError::Database(error.to_string()))?;
    connection.execute("DELETE FROM ai_cache", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::{open_pool, run_local_migrations};

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
}
