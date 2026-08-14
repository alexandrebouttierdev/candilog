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
    (6, include_str!("../../migrations/006_index_dates.sql")),
    (
        7,
        include_str!("../../migrations/007_lettres_motivation.sql"),
    ),
];

/// Version de schéma atteinte après application de toutes les migrations.
pub const DERNIERE_VERSION: i64 = 7;

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

/// Applique les migrations locales non encore jouées, chacune dans sa propre transaction.
///
/// Les migrations qui recréent une table (procédé imposé par `SQLite`, qui ne sait pas ajouter
/// de contrainte par `ALTER TABLE`) passent par `DROP TABLE`. Or, clés étrangères actives,
/// `SQLite` réalise un DELETE implicite avant de supprimer une table : les `ON DELETE CASCADE`
/// des tables enfants se déclenchent et effacent leur contenu, alors même que la table parente
/// est aussitôt recréée à l'identique.
///
/// On suit donc la procédure officielle de changement de schéma : `foreign_keys` désactivé
/// **hors** transaction (le `PRAGMA` est sans effet à l'intérieur), `foreign_key_check` avant
/// de valider, réactivation ensuite — y compris si la migration échoue, la connexion étant
/// rendue au pool.
///
/// # Errors
/// Retourne `AppError::Database` si une migration échoue ou laisse une référence pendante.
pub fn run_local_migrations(pool: &SqlitePool) -> AppResult<()> {
    let mut conn = pool.get().map_err(|e| AppError::Database(e.to_string()))?;
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let a_migrer: Vec<_> = MIGRATIONS.iter().filter(|(c, _)| *c > version).collect();
    if a_migrer.is_empty() {
        return Ok(());
    }

    tracing::info!(depuis = version, jusqu_a = DERNIERE_VERSION, "migration");
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
        verifier_integrite_referentielle(&transaction, *cible)?;
        transaction.pragma_update(None, "user_version", cible)?;
        transaction.commit()?;
        tracing::info!(version = cible, "migration appliquée");
    }
    Ok(())
}

/// Refuse de valider une migration qui laisserait une référence pendante.
///
/// Les clés étrangères étant désactivées le temps de la recréation des tables, ce contrôle
/// est le seul garde-fou : sans lui, une erreur de recopie passerait inaperçue jusqu'à la
/// première lecture.
fn verifier_integrite_referentielle(conn: &rusqlite::Connection, cible: i64) -> AppResult<()> {
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
#[path = "tests/db/mod.rs"]
mod tests;
