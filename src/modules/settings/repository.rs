//! Accès aux paramètres applicatifs (base locale `SQLite`, table singleton `parametres`).

use crate::modules::settings::model::AppSettings;
use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use crate::shared::sqlite::{connexion, maintenant_iso, traduire_erreur};
use rusqlite::OptionalExtension;

/// Contrat d'accès aux paramètres applicatifs (table singleton : une seule ligne, `id = 1`).
pub trait SettingsRepository: Send + Sync {
    /// Récupère les paramètres, ou les paramètres par défaut si aucune ligne n'existe encore
    /// ou si le contenu stocké est illisible.
    ///
    /// # Errors
    /// `AppError::Database` si la connexion ou la requête échoue.
    fn get(&self) -> AppResult<AppSettings>;
    /// Crée ou remplace la ligne unique des paramètres.
    ///
    /// # Errors
    /// `AppError::Serialization` si les paramètres ne peuvent pas être sérialisés ; sinon
    /// `AppError::Database`.
    fn upsert(&self, settings: &AppSettings) -> AppResult<AppSettings>;
}

/// Implémentation `SQLite` du dépôt de paramètres.
pub struct SqliteSettingsRepository {
    pool: SqlitePool,
}

impl SqliteSettingsRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl SettingsRepository for SqliteSettingsRepository {
    fn get(&self) -> AppResult<AppSettings> {
        let conn = connexion(&self.pool)?;
        let contenu_texte: Option<String> = conn
            .query_row("SELECT data FROM parametres WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|e| traduire_erreur(e, "parametres"))?;
        // Les paramètres sont lus au démarrage de l'application : contrairement au profil, un
        // JSON corrompu ne doit jamais empêcher l'application de s'ouvrir. On retombe donc
        // silencieusement sur les valeurs par défaut plutôt que de remonter une erreur — ce
        // repli est délibéré, pas une erreur avalée par négligence.
        Ok(contenu_texte
            .and_then(|texte| serde_json::from_str(&texte).ok())
            .unwrap_or_default())
    }

    fn upsert(&self, settings: &AppSettings) -> AppResult<AppSettings> {
        let conn = connexion(&self.pool)?;
        let maintenant = maintenant_iso();
        let contenu_texte =
            serde_json::to_string(settings).map_err(|e| AppError::Serialization(e.to_string()))?;
        conn.execute(
            "INSERT INTO parametres (id, data, updated_at) VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
            rusqlite::params![contenu_texte, maintenant],
        )
        .map_err(|e| traduire_erreur(e, "parametres"))?;
        Ok(settings.clone())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
