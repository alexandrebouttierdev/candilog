//! Logique métier des versions de CV.

use crate::modules::cv::model::{CvVersion, CvVersionSummary};
use crate::modules::cv::repository::CvVersionRepository;
use crate::shared::error::{AppError, AppResult};
use serde_json::Value;
use uuid::Uuid;

/// Longueur maximale d'un nom de version.
const NAME_MAX: usize = 120;

/// Service métier des versions de CV, générique sur le dépôt (testable via mock).
pub struct CvVersionService<R: CvVersionRepository> {
    repo: R,
}

impl<R: CvVersionRepository> CvVersionService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Valide le nom puis persiste une version.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom est vide ou trop long ; `AppError::Serialization` si le
    /// contenu ne peut pas être sérialisé ; sinon l'erreur du dépôt.
    pub fn save(&self, name: &str, content: &Value) -> AppResult<CvVersion> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation(
                "Le nom de la version est requis".into(),
            ));
        }
        if trimmed.chars().count() > NAME_MAX {
            return Err(AppError::Validation(
                "Le nom de la version est trop long (120 max)".into(),
            ));
        }
        self.repo.create(trimmed, content)
    }

    /// Liste les résumés des versions de CV (les plus récentes d'abord, sans le contenu).
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list(&self) -> AppResult<Vec<CvVersionSummary>> {
        self.repo.list()
    }

    /// Charge une version complète.
    ///
    /// # Errors
    /// `AppError::NotFound` si absente ; `AppError::Serialization` si le contenu stocké est
    /// invalide ; sinon l'erreur du dépôt.
    pub fn load(&self, id: Uuid) -> AppResult<CvVersion> {
        self.repo.get(id)
    }

    /// Supprime une version.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn delete(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
