//! Cas d'usage des contacts du réseau.

use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::contacts::domain::{Contact, ContactRepository, MajContact, NouveauContact};

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

    /// Liste tous les contacts.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister(&self) -> AppResult<Vec<Contact>> {
        self.repo.list()
    }

    /// Récupère un contact par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn obtenir(&self, id: uuid::Uuid) -> AppResult<Contact> {
        self.repo.get(id)
    }

    /// Charge une page du réseau sans matérialiser l'ensemble.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister_page(&self, page: u64, page_size: u64, search: &str) -> AppResult<Page<Contact>> {
        self.repo.list_page(page, page_size, search)
    }

    /// Valide puis crée le contact.
    ///
    /// # Errors
    /// `AppError::Validation` si le prénom ou le nom est vide, ou si le profil LinkedIn
    /// n'est pas une URL HTTP(S).
    pub fn creer(&self, input: &NouveauContact) -> AppResult<Contact> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide puis met à jour le contact.
    ///
    /// # Errors
    /// `AppError::Validation` si les champs requis manquent ;
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn modifier(&self, id: uuid::Uuid, input: &MajContact) -> AppResult<Contact> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Supprime un contact.
    ///
    /// # Errors
    /// `AppError::Validation` si des candidatures ou des entretiens le référencent.
    pub fn supprimer(&self, id: uuid::Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Règles communes à la création et à la modification.
    ///
    /// Ces contrôles doublent ceux du schéma Zod côté React : la validation frontend sert
    /// l'ergonomie, elle ne garantit rien — une commande Tauri est appelable sans passer par
    /// le formulaire (MIGRATION.md §14).
    fn valider(input: &NouveauContact) -> AppResult<()> {
        if input.prenom.trim().is_empty() || input.nom.trim().is_empty() {
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
