//! Pool de connexions `SQLite` locales et initialisation du schéma.
//!
//! Le schéma complet vit dans un seul fichier `init_schema.sql`, embarqué dans le binaire :
//! tables, index et semences des référentiels métier. Le curseur est `PRAGMA user_version` :
//! une base déjà à jour ne rejoue pas le fichier, et le rejouer resterait sans effet — tout y
//! est écrit en `CREATE IF NOT EXISTS` / `INSERT OR IGNORE`.

use crate::core::errors::{AppError, AppResult};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

/// Pool de connexions `SQLite` partagé par l'application.
pub type SqlitePool = Pool<SqliteConnectionManager>;

/// Versions de schéma, appliquées par ordre croissant.
///
/// Une seule entrée : `init_schema.sql` porte l'intégralité du modèle — tables, index et
/// semences des référentiels. Aucune migration héritée n'est conservée, une base neuve
/// obtient directement le schéma final.
const MIGRATIONS: &[(i64, &str)] = &[(1, include_str!("../../../migrations/init_schema.sql"))];

/// Version de schéma atteinte après application de `init_schema`.
pub const LATEST_SCHEMA_VERSION: i64 = 1;

/// Vérifie qu'un fichier existant appartient à la génération de schéma prise en charge.
///
/// L'inspection est strictement en lecture seule et doit précéder [`open_pool`] : ce dernier
/// active WAL et d'autres pragmas qui peuvent modifier le fichier. Un chemin absent ou un
/// fichier vide représente une nouvelle base ; une base non vide sans version est ambiguë et
/// donc refusée au même titre qu'une version supérieure au schéma courant.
///
/// # Errors
/// Retourne [`AppError::IncompatibleData`] pour une génération abandonnée, ou l'erreur SQLite
/// si le fichier n'est pas lisible comme une base.
pub fn validate_database_file(path: &Path) -> AppResult<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = std::fs::metadata(path)
        .map_err(|error| AppError::Database(format!("métadonnées inaccessibles : {error}")))?;
    if metadata.len() == 0 {
        return Ok(());
    }

    let connection =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let user_tables: i64 = connection.query_row(
        "SELECT count(*) FROM sqlite_master \
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        [],
        |row| row.get(0),
    )?;

    if version > LATEST_SCHEMA_VERSION || (version == 0 && user_tables > 0) {
        return Err(AppError::IncompatibleData(format!(
            "version de schéma {version}"
        )));
    }
    Ok(())
}

/// Applique les réglages indispensables à **chaque** connexion du pool.
///
/// `foreign_keys` est désactivé par défaut dans `SQLite` et se règle par connexion : sans cet
/// initialiseur, les clés étrangères seraient silencieusement ignorées.
///
/// `search_key` est enregistrée ici pour la même raison : une fonction scalaire vit dans la
/// connexion, pas dans le fichier de base. Elle donne à SQLite exactement la normalisation
/// que les dépôts appliquent au terme recherché — `lower()` n'agit que sur l'ASCII et
/// laisserait « ÉCOLE » hors de portée d'une recherche « école ».
fn init_connection(conn: &mut rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    register_search_key(conn)
}

/// Publie [`crate::core::utils::text::search_key`] comme fonction scalaire SQLite.
///
/// Déclarée déterministe : SQLite peut alors la sortir d'une boucle ou l'utiliser dans un
/// index partiel, et le résultat ne dépend que de son argument.
fn register_search_key(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.create_scalar_function(
        "search_key",
        1,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
        |context| {
            let value: Option<String> = context.get(0)?;
            Ok(value.map(|value| crate::core::utils::text::search_key(&value)))
        },
    )
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

    tracing::info!(from = version, jusqu_a = LATEST_SCHEMA_VERSION, "migration");
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
