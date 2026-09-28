//! Domaine de l'historique des relations.

pub mod history;
pub mod repository;

pub use history::{
    HistoryEntry, HistoryKind, NewRelationNote, RelationKind, RelationNote, RelationRef,
    MAX_HISTORY, MAX_NOTE_CHARS,
};
pub use repository::RelationHistoryRepository;
