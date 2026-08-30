//! Commandes Tauri des référentiels métier.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::referentials::domain::Referentials;
use std::sync::Arc;
use tauri::State;

/// Charge les quatre référentiels, dans l'ordre d'affichage des sélecteurs.
#[tauri::command(rename_all = "snake_case")]
pub async fn referentials_load(state: State<'_, AppState>) -> AppResult<Referentials> {
    let service = Arc::clone(&state.referentials);
    blocking::execute(move || service.load()).await
}
