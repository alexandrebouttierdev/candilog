//! Logique métier des entreprises (minimal : liste + création).

use crate::modules::entreprises::model::{Entreprise, MajEntreprise, NouvelleEntreprise};
use crate::modules::entreprises::repository::EntrepriseRepository;
use crate::modules::metriques::model::Page;
use crate::shared::error::{AppError, AppResult};
use crate::shared::validation::validate_optional_http_url;

/// Service métier des entreprises, générique sur le dépôt (testable via mock).
pub struct EntrepriseService<R: EntrepriseRepository> {
    repo: R,
}

impl<R: EntrepriseRepository> EntrepriseService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Liste les entreprises de l'utilisateur.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister(&self) -> AppResult<Vec<Entreprise>> {
        self.repo.list()
    }

    /// Récupère une entreprise par identifiant.
    pub fn obtenir(&self, id: uuid::Uuid) -> AppResult<Entreprise> {
        self.repo.get(id)
    }

    /// Charge une page filtrée sans matérialiser tout le répertoire.
    pub fn lister_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
        company_type: Option<&str>,
    ) -> AppResult<Page<Entreprise>> {
        self.repo.list_page(page, page_size, search, company_type)
    }

    /// Liste les types réellement disponibles pour le filtre du répertoire.
    pub fn lister_types(&self) -> AppResult<Vec<String>> {
        self.repo.list_types()
    }

    /// Valide le nom puis crée l'entreprise.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom est vide ; sinon l'erreur du dépôt.
    pub fn creer(&self, input: &NouvelleEntreprise) -> AppResult<Entreprise> {
        if input.nom.trim().is_empty() {
            return Err(AppError::Validation(
                "Le nom de l'entreprise est requis".into(),
            ));
        }
        validate_optional_http_url(input.site_web.as_deref(), "Le site web")?;
        self.repo.create(input)
    }

    /// Valide le nom puis met à jour l'entreprise.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom est vide ; sinon l'erreur du dépôt.
    pub fn modifier(&self, id: uuid::Uuid, input: &MajEntreprise) -> AppResult<Entreprise> {
        if input.nom.trim().is_empty() {
            return Err(AppError::Validation(
                "Le nom de l'entreprise est requis".into(),
            ));
        }
        validate_optional_http_url(input.site_web.as_deref(), "Le site web")?;
        self.repo.update(id, input)
    }

    /// Supprime une entreprise de l'utilisateur.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt (`AppError::Validation` si des candidatures sont liées).
    pub fn supprimer(&self, id: uuid::Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
