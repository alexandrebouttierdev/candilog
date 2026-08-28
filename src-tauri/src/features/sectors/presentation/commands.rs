//! Commands Tauri du référentiel des secteurs.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::sectors::domain::ActivitySector;
use std::sync::Arc;
use tauri::State;

/// List les secteurs d'activité, dans l'ordre d'affichage du sélecteur.
#[tauri::command(rename_all = "snake_case")]
pub async fn sectors_list(state: State<'_, AppState>) -> AppResult<Vec<ActivitySector>> {
    let service = Arc::clone(&state.sectors);
    blocking::execute(move || service.list()).await
}
