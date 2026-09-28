//! Commandes Tauri de l'historique des relations.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::relations::domain::{
    HistoryEntry, NewRelationNote, RelationNote, RelationRef,
};
use std::sync::Arc;
use tauri::State;

/// Historique d'une entreprise ou d'un contact, du plus récent au plus ancien.
#[tauri::command(rename_all = "snake_case")]
pub async fn relations_history(
    state: State<'_, AppState>,
    relation: RelationRef,
) -> AppResult<Vec<HistoryEntry>> {
    let service = Arc::clone(&state.relation_history);
    blocking::execute(move || service.history(relation)).await
}

/// Ajoute une note datée à l'historique d'une fiche.
#[tauri::command(rename_all = "snake_case")]
pub async fn relations_add_note(
    state: State<'_, AppState>,
    input: NewRelationNote,
) -> AppResult<RelationNote> {
    let service = Arc::clone(&state.relation_history);
    blocking::execute(move || service.add_note(&input)).await
}

/// Supprime une note de l'historique.
#[tauri::command(rename_all = "snake_case")]
pub async fn relations_delete_note(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.relation_history);
    blocking::execute(move || service.delete_note(id)).await
}
