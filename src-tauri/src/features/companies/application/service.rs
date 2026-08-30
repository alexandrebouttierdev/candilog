//! Cas d'usage des entreprises.

use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::companies::domain::{
    Company, CompanyFilter, CompanyRepository, CompanyUpdate, NewCompany,
};

/// Service métier des entreprises, générique sur le dépôt.
pub struct CompanyService<R: CompanyRepository> {
    repo: R,
}

impl<R: CompanyRepository> CompanyService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Liste toutes les entreprises, pour alimenter un sélecteur.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list(&self) -> AppResult<Vec<Company>> {
        self.repo.list()
    }

    /// Récupère une entreprise par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn get(&self, id: uuid::Uuid) -> AppResult<Company> {
        self.repo.get(id)
    }

    /// Renvoie une page du répertoire sans matérialiser l'ensemble.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list_page(
        &self,
        page: u64,
        page_size: u64,
        filter: &CompanyFilter,
    ) -> AppResult<Page<Company>> {
        self.repo.list_page(page, page_size, filter)
    }

    /// Valide puis crée l'entreprise.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom est vide ou le site web mal formé.
    pub fn create(&self, input: &NewCompany) -> AppResult<Company> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide puis met à jour l'entreprise.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom est vide ou le site web mal formé ;
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn update(&self, id: uuid::Uuid, input: &CompanyUpdate) -> AppResult<Company> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Supprime une entreprise.
    ///
    /// # Errors
    /// `AppError::Validation` si des candidatures y sont rattachées.
    pub fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Règles communes à la création et à la modification.
    ///
    /// Ces contrôles doublent ceux du schéma Zod côté React : la validation frontend sert
    /// l'ergonomie, elle ne garantit rien — une commande Tauri est appelable sans passer par
    /// le formulaire (MIGRATION.md §14).
    fn valider(input: &NewCompany) -> AppResult<()> {
        if input.name.trim().is_empty() {
            return Err(AppError::Validation(
                "Le nom de l'entreprise est requis".into(),
            ));
        }
        validate_optional_http_url(input.website.as_deref(), "Le site web")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
