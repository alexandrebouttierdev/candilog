//! Commandes Tauri du référentiel des secteurs.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::secteurs::domain::SecteurActivite;
use std::sync::Arc;
use tauri::State;

/// Liste les secteurs d'activité, dans l'ordre d'affichage du sélecteur.
#[tauri::command]
pub async fn secteurs_lister(state: State<'_, AppState>) -> AppResult<Vec<SecteurActivite>> {
    let service = Arc::clone(&state.secteurs);
    blocking::execute(move || service.lister()).await
}
