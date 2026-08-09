//! Sauvegarde et validation des bases SQLite Candilog.

use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use std::path::Path;

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
    let header = std::fs::read(path)
        .map_err(|error| AppError::Database(format!("Impossible de lire le backup : {error}")))?;
    if header.len() < 16 || &header[..16] != b"SQLite format 3\0" {
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
    Ok(())
}

/// Remplace atomiquement le contenu de la base active par un backup validé.
///
/// # Errors
/// Retourne une erreur si le backup est invalide ou si l'API backup échoue.
pub fn import(pool: &SqlitePool, source_path: &Path) -> AppResult<()> {
    validate(source_path)?;
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
    drop(backup);
    drop(target);
    crate::shared::db::run_local_migrations(pool)
}

/// Vide toutes les données utilisateur en conservant le schéma et ses migrations.
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

#[cfg(test)]
#[path = "tests/backup/mod.rs"]
mod tests;
