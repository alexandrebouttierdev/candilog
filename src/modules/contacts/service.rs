//! Logique métier des contacts (validation + délégation au dépôt).

use crate::modules::contacts::model::{Contact, MajContact, NouveauContact};
use crate::modules::contacts::repository::ContactRepository;
use crate::modules::metriques::model::Page;
use crate::shared::error::{AppError, AppResult};
use crate::shared::validation::validate_optional_http_url;

/// Service métier des contacts, générique sur le dépôt (testable via mock).
pub struct ContactService<R: ContactRepository> {
    repo: R,
}

impl<R: ContactRepository> ContactService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Liste les contacts (triés par nom puis prénom).
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister(&self) -> AppResult<Vec<Contact>> {
        self.repo.list()
    }

    /// Récupère un contact par identifiant.
    pub fn obtenir(&self, id: uuid::Uuid) -> AppResult<Contact> {
        self.repo.get(id)
    }

    /// Charge une page filtrée sans matérialiser tout le réseau.
    pub fn lister_page(&self, page: u64, page_size: u64, search: &str) -> AppResult<Page<Contact>> {
        self.repo.list_page(page, page_size, search)
    }

    /// Valide (prénom + nom requis) puis crée le contact.
    ///
    /// # Errors
    /// `AppError::Validation` si le prénom ou le nom est vide ; sinon l'erreur du dépôt.
    pub fn creer(&self, input: &NouveauContact) -> AppResult<Contact> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide (prénom + nom requis) puis met à jour le contact.
    ///
    /// # Errors
    /// `AppError::Validation` si le prénom ou le nom est vide ; sinon l'erreur du dépôt.
    pub fn modifier(&self, id: uuid::Uuid, input: &MajContact) -> AppResult<Contact> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Supprime un contact.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn supprimer(&self, id: uuid::Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Valide qu'un contact a un prénom et un nom non vides.
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
