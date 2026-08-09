//! Pool de connexions `SQLite` locales et migrations locales.
//!
//! Les migrations sont des fichiers `.sql` numérotés, embarqués dans le binaire et appliqués
//! par ordre croissant. Le curseur est `PRAGMA user_version` : une base déjà installée ne
//! rejoue que les migrations postérieures à sa version.

use crate::shared::error::{AppError, AppResult};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

/// Pool de connexions `SQLite` partagé par l'application.
pub type SqlitePool = Pool<SqliteConnectionManager>;

/// Migrations locales, appliquées par ordre croissant de version.
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../../migrations/001_tables_locales.sql")),
    (
        2,
        include_str!("../../migrations/002_purge_score_offre.sql"),
    ),
    (3, include_str!("../../migrations/003_drop_offres.sql")),
    (4, include_str!("../../migrations/004_schema_metier.sql")),
    (5, include_str!("../../migrations/005_contraintes_enum.sql")),
];

/// Version de schéma atteinte après application de toutes les migrations.
pub const DERNIERE_VERSION: i64 = 5;

/// Applique les réglages indispensables à **chaque** connexion du pool.
///
/// `foreign_keys` est désactivé par défaut dans `SQLite` et se règle par connexion : sans cet
/// initialiseur, les clés étrangères seraient silencieusement ignorées.
fn initialiser_connexion(conn: &mut rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(())
}

/// Ouvre un pool `SQLite`. `path = None` ouvre une base en mémoire, réservée aux tests :
/// si le répertoire de données est inaccessible, l'application ne démarre pas.
///
/// La base mémoire utilise une URI `cache=shared` nommée : avec le manager mémoire par défaut,
/// chaque connexion du pool ouvrirait sa propre base isolée, et toute lecture après écriture
/// via le pool échouerait.
///
/// # Errors
/// Retourne `AppError::Database` si le pool ne peut pas être construit.
pub fn open_pool(path: Option<&Path>) -> AppResult<SqlitePool> {
    let manager = match path {
        Some(p) => SqliteConnectionManager::file(p),
        None => SqliteConnectionManager::file(format!(
            "file:candilog-{}?mode=memory&cache=shared",
            uuid::Uuid::new_v4()
        ))
        .with_flags(
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
                | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        ),
    }
    .with_init(initialiser_connexion);
    // `min_idle(1)` garde une connexion vivante : une base mémoire partagée est détruite
    // dès que sa dernière connexion se ferme.
    Pool::builder()
        .min_idle(Some(1))
        .build(manager)
        .map_err(|e| AppError::Database(e.to_string()))
}

/// Applique les migrations locales non encore jouées, chacune dans sa propre transaction.
///
/// # Errors
/// Retourne `AppError::Database` si une migration échoue.
pub fn run_local_migrations(pool: &SqlitePool) -> AppResult<()> {
    let mut conn = pool.get().map_err(|e| AppError::Database(e.to_string()))?;
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    for (cible, sql) in MIGRATIONS {
        if *cible <= version {
            continue;
        }
        let transaction = conn.transaction()?;
        transaction.execute_batch(sql)?;
        transaction.pragma_update(None, "user_version", cible)?;
        transaction.commit()?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/db/mod.rs"]
mod tests;
