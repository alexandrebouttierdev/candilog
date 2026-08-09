//! Accès au profil (base locale `SQLite`, table singleton `profil`).

use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use crate::shared::profile::Profile;
use crate::shared::sqlite::{connexion, maintenant_iso, traduire_erreur};
use rusqlite::OptionalExtension;

/// Contrat d'accès au profil (table singleton : une seule ligne, `id = 1`).
pub trait ProfilRepository: Send + Sync {
    /// Récupère le profil, ou le profil par défaut si aucune ligne n'existe encore.
    ///
    /// # Errors
    /// `AppError::Serialization` si le contenu stocké est invalide ; sinon `AppError::Database`.
    fn get(&self) -> AppResult<Profile>;
    /// Crée ou remplace la ligne unique du profil.
    ///
    /// # Errors
    /// `AppError::Serialization` si le profil ne peut pas être sérialisé ; sinon
    /// `AppError::Database`.
    fn upsert(&self, profil: &Profile) -> AppResult<Profile>;
}

/// Implémentation `SQLite` du dépôt de profil.
pub struct SqliteProfilRepository {
    pool: SqlitePool,
}

impl SqliteProfilRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ProfilRepository for SqliteProfilRepository {
    fn get(&self) -> AppResult<Profile> {
        let conn = connexion(&self.pool)?;
        let contenu_texte: Option<String> = conn
            .query_row("SELECT data FROM profil WHERE id = 1", [], |row| row.get(0))
            .optional()
            .map_err(|e| traduire_erreur(e, "profil"))?;
        match contenu_texte {
            Some(texte) => {
                serde_json::from_str(&texte).map_err(|e| AppError::Serialization(e.to_string()))
            }
            None => Ok(Profile::default()),
        }
    }

    fn upsert(&self, profil: &Profile) -> AppResult<Profile> {
        let conn = connexion(&self.pool)?;
        let maintenant = maintenant_iso();
        let contenu_texte =
            serde_json::to_string(profil).map_err(|e| AppError::Serialization(e.to_string()))?;
        conn.execute(
            "INSERT INTO profil (id, data, updated_at) VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
            rusqlite::params![contenu_texte, maintenant],
        )
        .map_err(|e| traduire_erreur(e, "profil"))?;
        Ok(profil.clone())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
