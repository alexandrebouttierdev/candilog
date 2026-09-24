//! Commandes Tauri des vues enregistrées.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::views::domain::{NewSavedView, SavedView};
use std::sync::Arc;
use tauri::State;

/// Liste les vues, dans l'ordre de la navigation.
#[tauri::command(rename_all = "snake_case")]
pub async fn saved_views_list(state: State<'_, AppState>) -> AppResult<Vec<SavedView>> {
    let service = Arc::clone(&state.views);
    blocking::execute(move || service.list()).await
}

/// Enregistre une vue.
#[tauri::command(rename_all = "snake_case")]
pub async fn saved_views_create(
    state: State<'_, AppState>,
    input: NewSavedView,
) -> AppResult<SavedView> {
    let service = Arc::clone(&state.views);
    blocking::execute(move || service.create(&input)).await
}

/// Renomme une vue ou remplace son filtre.
#[tauri::command(rename_all = "snake_case")]
pub async fn saved_views_update(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    input: NewSavedView,
) -> AppResult<SavedView> {
    let service = Arc::clone(&state.views);
    blocking::execute(move || service.update(id, &input)).await
}

/// Duplique une vue.
#[tauri::command(rename_all = "snake_case")]
pub async fn saved_views_duplicate(
    state: State<'_, AppState>,
    id: uuid::Uuid,
) -> AppResult<SavedView> {
    let service = Arc::clone(&state.views);
    blocking::execute(move || service.duplicate(id)).await
}

/// Supprime une vue.
#[tauri::command(rename_all = "snake_case")]
pub async fn saved_views_delete(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.views);
    blocking::execute(move || service.delete(id)).await
}
