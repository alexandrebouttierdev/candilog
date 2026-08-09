//! Cache local (`SQLite`) des résultats d'analyse `IA`.
//!
//! Ré-analyser le même CV/offre avec le même modèle recoûte, sur un modèle local, plusieurs
//! secondes à plusieurs minutes. Ce cache mémorise le résultat `JSON` d'une opération `LLM`,
//! indexé par une clé dérivée du **fournisseur + modèle + mode + opération + texte d'entrée**.
//! Un changement de l'un de ces éléments produit une clé différente : invalidation naturelle,
//! aucune donnée périmée servie.
//!
//! Le cache vit au **niveau commande** (il a besoin du pool `SQLite`), ce qui garde
//! `CvEngine`/service purs — conforme à l'architecture en couches du projet.

use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use sha2::{Digest, Sha256};

/// Entrée à écrire dans le cache `IA`.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Clé de cache (voir [`cache_key`]).
    pub cle: String,
    /// Résultat `JSON` sérialisé.
    pub valeur: String,
    /// Nom du fournisseur (traçabilité/purge).
    pub provider: String,
    /// Nom du modèle (traçabilité/purge).
    pub modele: String,
    /// Opération `LLM` (traçabilité/purge).
    pub operation: String,
    /// Horodatage de création (`RFC 3339`).
    pub cree_le: String,
}

/// Calcule la clé de cache stable d'une opération `LLM`.
///
/// `sha256` hexadécimal de `provider | model | mode | operation | input`. Stable entre
/// exécutions (contrairement à un hash de la bibliothèque standard), donc utilisable comme
/// clé persistante.
#[must_use]
pub fn cache_key(provider: &str, model: &str, mode: &str, operation: &str, input: &str) -> String {
    let mut hasher = Sha256::new();
    for part in [provider, model, mode, operation, input] {
        hasher.update(part.as_bytes());
        hasher.update([0]); // séparateur : évite les collisions par concaténation ambiguë
    }
    format!("{:x}", hasher.finalize())
}

/// Contrat de persistance du cache `IA`.
pub trait CacheIaRepository: Send + Sync {
    /// Récupère la valeur `JSON` associée à `cle`, ou `None` si absente.
    ///
    /// # Errors
    /// `AppError::Database` si la lecture échoue.
    fn get(&self, cle: &str) -> AppResult<Option<String>>;

    /// Insère (ou remplace) une entrée de cache.
    ///
    /// # Errors
    /// `AppError::Database` si l'écriture échoue.
    fn put(&self, entry: &CacheEntry) -> AppResult<()>;

    /// Vide entièrement le cache `IA`.
    ///
    /// # Errors
    /// `AppError::Database` si la requête échoue.
    fn reset(&self) -> AppResult<()>;
}

/// Implémentation `SQLite` du cache `IA`.
pub struct SqliteCacheIaRepository {
    pool: SqlitePool,
}

impl SqliteCacheIaRepository {
    /// Construit le dépôt à partir du pool `SQLite` partagé.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl CacheIaRepository for SqliteCacheIaRepository {
    fn get(&self, cle: &str) -> AppResult<Option<String>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT valeur FROM cache_ia WHERE cle = ?1")?;
        let mut rows = stmt.query_map([cle], |r| r.get::<_, String>(0))?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    fn put(&self, entry: &CacheEntry) -> AppResult<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO cache_ia (cle, valeur, provider, modele, operation, cree_le)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                entry.cle,
                entry.valeur,
                entry.provider,
                entry.modele,
                entry.operation,
                entry.cree_le,
            ],
        )?;
        Ok(())
    }

    fn reset(&self) -> AppResult<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute("DELETE FROM cache_ia", [])?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/cache/mod.rs"]
mod tests;
