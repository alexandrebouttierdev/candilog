//! Validation et orchestration de l'historique des relations.

use crate::core::errors::{AppError, AppResult};
use crate::features::relations::domain::{
    HistoryEntry, NewRelationNote, RelationHistoryRepository, RelationNote, RelationRef,
    MAX_HISTORY, MAX_NOTE_CHARS,
};
use uuid::Uuid;

/// Service de l'historique : l'entrée IPC est revalidée ici, jamais crue sur parole.
pub struct RelationHistoryService<R: RelationHistoryRepository> {
    repo: R,
}

impl<R: RelationHistoryRepository> RelationHistoryService<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// `NotFound` si la fiche n'existe pas.
    pub fn history(&self, relation: RelationRef) -> AppResult<Vec<HistoryEntry>> {
        self.repo.history(relation, MAX_HISTORY)
    }

    /// # Errors
    /// `AppError::Validation` si la note est vide, trop longue ou mal datée ; `NotFound` si
    /// la fiche n'existe pas.
    pub fn add_note(&self, input: &NewRelationNote) -> AppResult<RelationNote> {
        let body = input.body.trim();
        if body.is_empty() {
            return Err(AppError::Validation("La note est vide.".into()));
        }
        if body.chars().count() > MAX_NOTE_CHARS {
            return Err(AppError::Validation(format!(
                "Une note tient en {MAX_NOTE_CHARS} caractères."
            )));
        }
        if chrono::NaiveDate::parse_from_str(&input.noted_on, "%Y-%m-%d").is_err() {
            return Err(AppError::Validation(
                "La date de la note est invalide (AAAA-MM-JJ attendu).".into(),
            ));
        }
        self.repo.add_note(&NewRelationNote {
            relation: input.relation,
            body: body.to_owned(),
            noted_on: input.noted_on.clone(),
        })
    }

    /// # Errors
    /// `NotFound` si la note n'existe pas.
    pub fn delete_note(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete_note(id)
    }
}
