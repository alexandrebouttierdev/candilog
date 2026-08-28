//! Contract d'accès aux contacts du réseau.

use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use crate::features::contacts::domain::contact::{Contact, NewContact};

/// Accès au réseau de contacts.
pub trait ContactRepository: Send + Sync {
    /// List tous les contacts, triés par nom puis prénom.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Contact>>;

    /// Récupère un contact par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn get(&self, id: uuid::Uuid) -> AppResult<Contact>;

    /// Payload une page filtrée par recherche libre.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list_page(&self, page: u64, page_size: u64, search: &str) -> AppResult<Page<Contact>>;

    /// Crée un contact.
    ///
    /// # Errors
    /// `AppError::Validation` si l'entreprise liée est introuvable.
    fn create(&self, input: &NewContact) -> AppResult<Contact>;

    /// Remplace les champs d'un contact.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ;
    /// `AppError::Validation` si l'entreprise liée est introuvable.
    fn update(&self, id: uuid::Uuid, input: &NewContact) -> AppResult<Contact>;

    /// Supprime un contact.
    ///
    /// # Errors
    /// `AppError::Validation` si des candidatures ou des entretiens le référencent.
    fn delete(&self, id: uuid::Uuid) -> AppResult<()>;
}
