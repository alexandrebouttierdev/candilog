//! Commands Tauri des relances.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::followups::domain::{FollowUp, NewFollowUp};
use std::sync::Arc;
use tauri::State;

/// Liste les relances d'une plage de dates, bornes incluses.
#[tauri::command(rename_all = "snake_case")]
pub async fn follow_ups_list_between(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> AppResult<Vec<FollowUp>> {
    let service = Arc::clone(&state.followups);
    blocking::execute(move || service.list_between(&from, &to)).await
}

/// Crée une relance.
#[tauri::command(rename_all = "snake_case")]
pub async fn follow_ups_create(
    state: State<'_, AppState>,
    input: NewFollowUp,
) -> AppResult<FollowUp> {
    let service = Arc::clone(&state.followups);
    blocking::execute(move || service.create(&input)).await
}

/// Remplace les champs d'une relance.
#[tauri::command(rename_all = "snake_case")]
pub async fn follow_ups_update(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    input: NewFollowUp,
) -> AppResult<FollowUp> {
    let service = Arc::clone(&state.followups);
    blocking::execute(move || service.update(id, &input)).await
}

/// Marque une relance faite (`done`) ou la rouvre.
#[tauri::command(rename_all = "snake_case")]
pub async fn follow_ups_set_done(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    done: bool,
) -> AppResult<FollowUp> {
    let service = Arc::clone(&state.followups);
    blocking::execute(move || service.set_done(id, done)).await
}

/// Supprime une relance.
#[tauri::command(rename_all = "snake_case")]
pub async fn follow_ups_delete(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.followups);
    blocking::execute(move || service.delete(id)).await
}
