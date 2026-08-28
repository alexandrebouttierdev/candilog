//! Sauvegarde et validation des bases SQLite Candilog.

use crate::core::database::{run_local_migrations, SqlitePool, DERNIERE_VERSION};
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
    let mut fichier = std::fs::File::open(path)
        .map_err(|error| AppError::Database(format!("Impossible de lire le backup : {error}")))?;
    let trop_court = matches!(
        fichier.read_exact(&mut entete),
        Err(ref error) if error.kind() == std::io::ErrorKind::UnexpectedEof
    );
    if trop_court || &entete != b"SQLite format 3\0" {
        return Err(AppError::Validation(
            "Le fichier sélectionné n'est pas une base SQLite.".into(),
        ));
    }
    let connection =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(AppError::Validation(format!(
            "Le backup SQLite est corrompu : {integrity}"
        )));
    }
    for table in [
        "candidatures",
        "entreprises",
        "contacts",
        "parametres",
        "profil",
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
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version > DERNIERE_VERSION {
        return Err(AppError::Validation(format!(
            "Ce backup a été créé par une version plus récente de Candilog (schéma {version}, \
             cette version gère le schéma {DERNIERE_VERSION}). Installez la mise à jour de \
             Candilog pour pouvoir le restaurer."
        )));
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
    let secours = chemin_de_secours(db_path);
    export(pool, &secours)?;
    tracing::info!("copie de secours prise avant restauration");

    let Err(echec) = remplacer(pool, source_path) else {
        tracing::info!("backup restauré");
        return Ok(());
    };
    tracing::error!(erreur = %echec, "restauration échouée, retour arrière");

    match remplacer(pool, &secours) {
        Ok(()) => Err(AppError::Validation(
            "Le backup n'a pas pu être restauré. La base d'origine a été restaurée : vos \
             données sont intactes."
                .into(),
        )),
        Err(perte) => {
            tracing::error!(erreur = %perte, "retour arrière échoué");
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

fn chemin_de_secours(db_path: &Path) -> PathBuf {
    if db_path.as_os_str().is_empty() {
        std::env::temp_dir().join(format!("candilog-secours-{}.sqlite", uuid::Uuid::new_v4()))
    } else {
        db_path.with_extension("sqlite.bak")
    }
}

/// Vide les données utilisateur en conservant le schéma, les migrations et le référentiel
/// des secteurs.
///
/// # Errors
/// Retourne une erreur si la transaction SQLite échoue.
pub fn reset_data(pool: &SqlitePool) -> AppResult<()> {
    let mut connection = pool
        .get()
        .map_err(|error| AppError::Database(error.to_string()))?;
    let transaction = connection.transaction()?;
    transaction.execute_batch(
        "DELETE FROM relances;
         DELETE FROM entretiens;
         DELETE FROM statut_history;
         DELETE FROM candidatures;
         DELETE FROM contacts;
         DELETE FROM entreprises;
         DELETE FROM cv_versions;
         DELETE FROM lettres_motivation;
         DELETE FROM profil;
         DELETE FROM parametres;
         DELETE FROM llm_appels;
         DELETE FROM scores_ats;
         DELETE FROM cache_ia;
         DELETE FROM app_kv;",
    )?;
    transaction.commit()?;
    Ok(())
}

/// Vide uniquement le cache des réponses IA.
///
/// # Errors
/// Retourne une erreur si la table ne peut pas être vidée.
pub fn vider_cache_ia(pool: &SqlitePool) -> AppResult<()> {
    let connection = pool
        .get()
        .map_err(|error| AppError::Database(error.to_string()))?;
    connection.execute("DELETE FROM cache_ia", [])?;
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
    fn export_produit_une_base_candilog_valide() {
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("copie.sqlite");
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        export(&pool, &destination).unwrap();
        validate(&destination).unwrap();
    }

    #[test]
    fn reset_conserve_le_referentiel_des_secteurs() {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        {
            let conn = pool.get().unwrap();
            conn.execute(
                "INSERT INTO app_kv (cle, valeur) VALUES ('marque', 'oui')",
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
        let secteurs: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'secteurs_activite'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(secteurs, 1);
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
                    "INSERT INTO entreprises (id, nom, created_at, updated_at)
                        VALUES ('e1', 'Acme', '2026-01-01', '2026-01-01');
                     INSERT INTO candidatures
                        (id, entreprise_id, poste, type_contrat, statut, date_envoi, created_at, updated_at)
                        VALUES ('c1', 'e1', 'Dev', 'CDI', 'EN_ATTENTE', '2026-01-01', '2026-01-01', '2026-01-01');",
                )
                .unwrap();
        }

        {
            let connection = rusqlite::Connection::open(&source_path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE candidatures (id TEXT PRIMARY KEY);
                     CREATE TABLE entreprises (id TEXT PRIMARY KEY);
                     CREATE TABLE contacts (id TEXT PRIMARY KEY);
                     CREATE TABLE parametres (id INTEGER PRIMARY KEY);
                     CREATE TABLE profil (id INTEGER PRIMARY KEY);",
                )
                .unwrap();
        }

        let erreur = import(&pool, &db_path, &source_path).unwrap_err();
        assert!(
            erreur.to_string().contains("restaurée"),
            "l'échec doit signaler que la base d'origine a été remise en place : {erreur}"
        );

        let connection = pool.get().unwrap();
        let poste: String = connection
            .query_row(
                "SELECT poste FROM candidatures WHERE id = 'c1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(poste, "Dev");
    }
}
