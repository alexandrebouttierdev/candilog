//! Contrat d'accès à l'historique des relations.

use crate::core::errors::AppResult;
use crate::features::relations::domain::history::{
    HistoryEntry, NewRelationNote, RelationNote, RelationRef,
};
use uuid::Uuid;

/// Accès à l'historique et aux notes des relations.
pub trait RelationHistoryRepository: Send + Sync {
    /// Historique de la fiche, du plus récent au plus ancien, borné à `limit` entrées.
    ///
    /// # Errors
    /// `AppError::NotFound` si la fiche n'existe pas ; `AppError::Database` sinon.
    fn history(&self, relation: RelationRef, limit: usize) -> AppResult<Vec<HistoryEntry>>;

    /// Enregistre une note sur la fiche.
    ///
    /// # Errors
    /// `AppError::NotFound` si la fiche n'existe pas ; `AppError::Database` sinon.
    fn add_note(&self, input: &NewRelationNote) -> AppResult<RelationNote>;

    /// Supprime une note.
    ///
    /// # Errors
    /// `AppError::NotFound` si la note n'existe pas.
    fn delete_note(&self, id: Uuid) -> AppResult<()>;
}
