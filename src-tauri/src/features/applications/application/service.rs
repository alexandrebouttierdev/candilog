//! Cas d'usage des candidatures.

use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::applications::domain::{
    Application, ApplicationRepository, ApplicationFilter, NewApplication,
    PipelineBreakdown, ApplicationStatus,
};
use uuid::Uuid;

/// Service métier des candidatures, générique sur le dépôt.
pub struct ApplicationService<R: ApplicationRepository> {
    repo: R,
}

impl<R: ApplicationRepository> ApplicationService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// List toutes les candidatures.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list(&self) -> AppResult<Vec<Application>> {
        self.repo.list()
    }

    /// Récupère une candidature par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn get(&self, id: Uuid) -> AppResult<Application> {
        self.repo.get(id)
    }

    /// Payload une page filtrée et triée.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list_page(
        &self,
        page: u64,
        page_size: u64,
        filter: &ApplicationFilter,
    ) -> AppResult<Page<Application>> {
        self.repo.list_page(page, page_size, filter)
    }

    /// Report les candidatures par statut, pour les en-têtes de colonnes du Kanban.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn breakdown(&self, filter: &ApplicationFilter) -> AppResult<PipelineBreakdown> {
        self.repo.breakdown(filter)
    }

    /// Valide puis crée la candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si le poste est vide, la date invalide ou le lien mal formé.
    pub fn create(&self, input: &NewApplication) -> AppResult<Application> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide puis met à jour la candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si un champ est invalide ; `AppError::NotFound` si
    /// l'identifiant est inconnu.
    pub fn update(&self, id: Uuid, input: &NewApplication) -> AppResult<Application> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Change le statut d'une candidature — le geste du glisser-déposer du Kanban.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn change_status(&self, id: Uuid, status: ApplicationStatus) -> AppResult<Application> {
        self.repo.update_status(id, status)
    }

    /// Supprime une candidature.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn delete(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Règles communes à la création et à la modification.
    ///
    /// La date est comparée au format `AAAA-MM-JJ` attendu par les requêtes de plage : une
    /// date stockée dans un autre format ferait échouer silencieusement les filtres de
    /// période, qui comparent des chaînes.
    fn valider(input: &NewApplication) -> AppResult<()> {
        if input.job_title.trim().is_empty() {
            return Err(AppError::Validation("Le poste est requis".into()));
        }
        if chrono::NaiveDate::parse_from_str(&input.sent_date, "%Y-%m-%d").is_err() {
            return Err(AppError::Validation("La date d'envoi est invalide".into()));
        }
        validate_optional_http_url(input.job_url.as_deref(), "Le lien de l'offre")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
