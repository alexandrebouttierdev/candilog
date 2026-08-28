//! Cas d'usage des entreprises.

use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::entreprises::domain::{
    Entreprise, EntrepriseRepository, MajEntreprise, NouvelleEntreprise,
};

/// Service métier des entreprises, générique sur le dépôt.
pub struct EntrepriseService<R: EntrepriseRepository> {
    repo: R,
}

impl<R: EntrepriseRepository> EntrepriseService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Liste toutes les entreprises, pour alimenter un sélecteur.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister(&self) -> AppResult<Vec<Entreprise>> {
        self.repo.list()
    }

    /// Récupère une entreprise par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn obtenir(&self, id: uuid::Uuid) -> AppResult<Entreprise> {
        self.repo.get(id)
    }

    /// Charge une page du répertoire sans matérialiser l'ensemble.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
        company_type: Option<&str>,
    ) -> AppResult<Page<Entreprise>> {
        self.repo.list_page(page, page_size, search, company_type)
    }

    /// Liste les types réellement présents, pour le filtre du répertoire.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister_types(&self) -> AppResult<Vec<String>> {
        self.repo.list_types()
    }

    /// Valide puis crée l'entreprise.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom est vide ou le site web mal formé.
    pub fn creer(&self, input: &NouvelleEntreprise) -> AppResult<Entreprise> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide puis met à jour l'entreprise.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom est vide ou le site web mal formé ;
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn modifier(&self, id: uuid::Uuid, input: &MajEntreprise) -> AppResult<Entreprise> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Supprime une entreprise.
    ///
    /// # Errors
    /// `AppError::Validation` si des candidatures y sont rattachées.
    pub fn supprimer(&self, id: uuid::Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Règles communes à la création et à la modification.
    ///
    /// Ces contrôles doublent ceux du schéma Zod côté React : la validation frontend sert
    /// l'ergonomie, elle ne garantit rien — une commande Tauri est appelable sans passer par
    /// le formulaire (MIGRATION.md §14).
    fn valider(input: &NouvelleEntreprise) -> AppResult<()> {
        if input.nom.trim().is_empty() {
            return Err(AppError::Validation(
                "Le nom de l'entreprise est requis".into(),
            ));
        }
        validate_optional_http_url(input.site_web.as_deref(), "Le site web")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
