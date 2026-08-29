//! Pool de connexions `SQLite` locales et initialisation du schéma.
//!
//! Le schéma complet vit dans un seul fichier `init_schema.sql`, embarqué dans le binaire.
//! Le curseur est `PRAGMA user_version` : une base déjà à jour ne rejoue pas le fichier.

use crate::core::errors::{AppError, AppResult};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

/// Pool de connexions `SQLite` partagé par l'application.
pub type SqlitePool = Pool<SqliteConnectionManager>;

/// Versions de schéma, appliquées par ordre croissant.
const MIGRATIONS: &[(i64, &str)] = &[(1, include_str!("../../../migrations/init_schema.sql"))];

/// Version de schéma atteinte après application de `init_schema`.
pub const DERNIERE_VERSION: i64 = 1;

/// Applique les réglages indispensables à **chaque** connexion du pool.
///
/// `foreign_keys` est désactivé par défaut dans `SQLite` et se règle par connexion : sans cet
/// initialiseur, les clés étrangères seraient silencieusement ignorées.
fn init_connection(conn: &mut rusqlite::Connection) -> rusqlite::Result<()> {
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
    .with_init(init_connection);
    // `min_idle(1)` garde une connexion vivante : une base mémoire partagée est détruite
    // dès que sa dernière connexion se ferme.
    //
    // `connection_timeout` redéfinit le défaut de r2d2 (30 s). `build()` étant bloquant et
    // appelé avant que la première fenêtre ne soit rendue, ce défaut laisserait l'utilisateur
    // devant un écran vide une demi-minute lorsque la base est illisible ou verrouillée.
    Pool::builder()
        .min_idle(Some(1))
        .connection_timeout(std::time::Duration::from_secs(2))
        .build(manager)
        .map_err(|e| AppError::Database(e.to_string()))
}

/// Applique le schéma local s'il n'est pas encore à jour, dans une transaction.
///
/// # Errors
/// Retourne `AppError::Database` si l'initialisation échoue ou laisse une référence pendante.
pub fn run_local_migrations(pool: &SqlitePool) -> AppResult<()> {
    let mut conn = pool.get().map_err(|e| AppError::Database(e.to_string()))?;
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let a_migrer: Vec<_> = MIGRATIONS.iter().filter(|(c, _)| *c > version).collect();
    if a_migrer.is_empty() {
        return Ok(());
    }

    tracing::info!(from = version, jusqu_a = DERNIERE_VERSION, "migration");
    conn.pragma_update(None, "foreign_keys", "OFF")?;
    let resultat = appliquer(&mut conn, &a_migrer);
    // Restaure l'état attendu par le reste de l'application avant de rendre la connexion,
    // que la migration ait abouti ou non.
    let reactivation = conn.pragma_update(None, "foreign_keys", "ON");
    resultat?;
    reactivation?;
    Ok(())
}

/// Corps de `run_local_migrations`, exécuté clés étrangères désactivées.
fn appliquer(conn: &mut rusqlite::Connection, a_migrer: &[&(i64, &str)]) -> AppResult<()> {
    for (cible, sql) in a_migrer {
        let transaction = conn.transaction()?;
        transaction.execute_batch(sql)?;
        check_integrite_referentielle(&transaction, *cible)?;
        transaction.pragma_update(None, "user_version", cible)?;
        transaction.commit()?;
        tracing::info!(version = cible, "migration appliquée");
    }
    Ok(())
}

/// Refuse de valider une migration qui laisserait une référence pendante.
fn check_integrite_referentielle(conn: &rusqlite::Connection, cible: i64) -> AppResult<()> {
    let violations: i64 =
        conn.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })?;
    if violations > 0 {
        tracing::error!(version = cible, violations, "migration incohérente");
        return Err(AppError::Database(format!(
            "la migration {cible} laisse {violations} référence(s) pendante(s)"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/connection/mod.rs"]
mod tests;
