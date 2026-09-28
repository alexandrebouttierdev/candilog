//! Contrat d'accès aux vues enregistrées.

use crate::core::errors::AppResult;
use crate::features::views::domain::saved_view::{NewSavedView, SavedView};
use uuid::Uuid;

/// Accès aux vues enregistrées.
pub trait SavedViewRepository: Send + Sync {
    /// Toutes les vues, dans l'ordre de la navigation.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<SavedView>>;

    /// Crée une vue en dernière position.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si l'écriture échoue.
    fn create(&self, input: &NewSavedView) -> AppResult<SavedView>;

    /// Remplace le nom et le filtre d'une vue.
    ///
    /// # Errors
    /// `AppError::NotFound` si la vue n'existe pas.
    fn update(&self, id: Uuid, input: &NewSavedView) -> AppResult<SavedView>;

    /// Relit une vue.
    ///
    /// # Errors
    /// `AppError::NotFound` si la vue n'existe pas.
    fn get(&self, id: Uuid) -> AppResult<SavedView>;

    /// Supprime une vue.
    ///
    /// # Errors
    /// `AppError::NotFound` si la vue n'existe pas.
    fn delete(&self, id: Uuid) -> AppResult<()>;
}
