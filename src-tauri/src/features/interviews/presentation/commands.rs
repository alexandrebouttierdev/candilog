//! Commands Tauri des entretiens.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::interviews::domain::{Interview, NewInterview};
use std::sync::Arc;
use tauri::State;

/// List les entretiens d'une plage de dates, bornes incluses.
#[tauri::command(rename_all = "snake_case")]
pub async fn interviews_list_between(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> AppResult<Vec<Interview>> {
    let service = Arc::clone(&state.interviews);
    blocking::execute(move || service.list_between(&from, &to)).await
}

/// Récupère un entretien par identifiant.
#[tauri::command(rename_all = "snake_case")]
pub async fn interviews_get(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<Interview> {
    let service = Arc::clone(&state.interviews);
    blocking::execute(move || service.get(id)).await
}

/// Enregistre un entretien et fait passer sa candidature au statut « Interview ».
///
/// `id` absent crée, `id` présent modifie : le chemin est unique côté dépôt, où l'écriture
/// et la mise à jour du statut sont dans la même transaction.
#[tauri::command(rename_all = "snake_case")]
pub async fn interviews_save(
    state: State<'_, AppState>,
    id: Option<uuid::Uuid>,
    input: NewInterview,
) -> AppResult<Interview> {
    let service = Arc::clone(&state.interviews);
    blocking::execute(move || service.save(id, &input)).await
}

/// Supprime un entretien.
#[tauri::command(rename_all = "snake_case")]
pub async fn interviews_delete(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.interviews);
    blocking::execute(move || service.delete(id)).await
}
