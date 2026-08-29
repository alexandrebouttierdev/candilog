//! Cas d'usage des contacts du réseau.

use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::contacts::domain::{Contact, ContactRepository, ContactUpdate, NewContact};

/// Service métier des contacts, générique sur le dépôt.
pub struct ContactService<R: ContactRepository> {
    repo: R,
}

impl<R: ContactRepository> ContactService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// List tous les contacts.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list(&self) -> AppResult<Vec<Contact>> {
        self.repo.list()
    }

    /// Récupère un contact par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn get(&self, id: uuid::Uuid) -> AppResult<Contact> {
        self.repo.get(id)
    }

    /// Payload une page du réseau filtrée par recherche et par rôle, sans matérialiser l'ensemble.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
        tracking_role: Option<&str>,
    ) -> AppResult<Page<Contact>> {
        self.repo.list_page(page, page_size, search, tracking_role)
    }

    /// Valide puis crée le contact.
    ///
    /// # Errors
    /// `AppError::Validation` si le prénom ou le nom est vide, ou si le profil LinkedIn
    /// n'est pas une URL HTTP(S).
    pub fn create(&self, input: &NewContact) -> AppResult<Contact> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide puis met à jour le contact.
    ///
    /// # Errors
    /// `AppError::Validation` si les champs requis manquent ;
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn update(&self, id: uuid::Uuid, input: &ContactUpdate) -> AppResult<Contact> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Supprime un contact.
    ///
    /// # Errors
    /// `AppError::Validation` si des candidatures ou des entretiens le référencent.
    pub fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Règles communes à la création et à la modification.
    ///
    /// Ces contrôles doublent ceux du schéma Zod côté React : la validation frontend sert
    /// l'ergonomie, elle ne garantit rien — une commande Tauri est appelable sans passer par
    /// le formulaire (MIGRATION.md §14).
    fn valider(input: &NewContact) -> AppResult<()> {
        if input.first_name.trim().is_empty() || input.name.trim().is_empty() {
            return Err(AppError::Validation(
                "Le prénom et le nom du contact sont requis".into(),
            ));
        }
        validate_optional_http_url(input.linkedin.as_deref(), "Le profil LinkedIn")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
