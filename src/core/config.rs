//! Résolution multiplateforme des chemins de données Candilog.

use crate::shared::error::{AppError, AppResult};
use std::path::PathBuf;

/// Chemins persistants utilisés par Candilog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// Dossier de données de l'application.
    pub data_dir: PathBuf,
    /// Base SQLite compatible avec Candilog Desktop historique.
    pub database: PathBuf,
    /// Dossier d'exports utilisateur.
    pub exports_dir: PathBuf,
}

impl AppPaths {
    /// Résout et crée les dossiers applicatifs sur Linux, Windows et macOS.
    ///
    /// # Errors
    /// Retourne une erreur si le système ne fournit pas de dossier de données ou si sa création échoue.
    pub fn discover() -> AppResult<Self> {
        // Un binaire de développement ne doit jamais ouvrir la base utilisateur historique.
        // Il écrit sous le projet ; les releases gardent le dossier historique.
        let data_dir = if let Some(override_dir) = std::env::var_os("CANDILOG_DATA_DIR") {
            PathBuf::from(override_dir)
        } else if cfg!(debug_assertions) {
            std::env::current_dir()
                .map_err(|error| {
                    AppError::Validation(format!(
                        "Le dossier courant de développement est introuvable : {error}"
                    ))
                })?
                .join(".candilog-dev")
        } else {
            dirs::data_dir()
                .ok_or_else(|| {
                    AppError::Validation("Le dossier de données du système est introuvable.".into())
                })?
                .join("com.candilog.desktop")
        };
        let exports_dir = data_dir.join("exports");
        std::fs::create_dir_all(&exports_dir).map_err(|error| {
            AppError::Database(format!("Impossible de créer le dossier Candilog : {error}"))
        })?;
        Ok(Self {
            database: data_dir.join("candilog.sqlite"),
            data_dir,
            exports_dir,
        })
    }

    /// Construit des chemins isolés, notamment pour les tests.
    #[must_use]
    pub fn in_directory(data_dir: PathBuf) -> Self {
        Self {
            database: data_dir.join("candilog.sqlite"),
            exports_dir: data_dir.join("exports"),
            data_dir,
        }
    }
}

#[cfg(test)]
#[path = "tests/config/mod.rs"]
mod tests;
