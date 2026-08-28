//! Commands Tauri du tableau de bord et des analyses.

use crate::app::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::utils::blocking;
use crate::features::analytics::domain::{Analytics, Period, Dashboard};
use std::sync::Arc;
use tauri::State;

/// Payload le tableau de bord à la date locale courante.
#[tauri::command]
pub async fn analyses_dashboard(state: State<'_, AppState>) -> AppResult<Dashboard> {
    let service = Arc::clone(&state.analytics);
    blocking::execute(move || service.dashboard(chrono::Local::now().date_naive())).await
}

/// Payload l'écran Analytics pour la période choisie.
#[tauri::command]
pub async fn analytics_load(state: State<'_, AppState>, period: Period) -> AppResult<Analytics> {
    let service = Arc::clone(&state.analytics);
    blocking::execute(move || service.analytics(period, chrono::Local::now().date_naive())).await
}

/// Écrit l'export CSV des analyses au chemin choisi dans le sélecteur natif.
#[tauri::command]
pub async fn analytics_export_csv(
    state: State<'_, AppState>,
    period: Period,
    path: String,
) -> AppResult<()> {
    let service = Arc::clone(&state.analytics);
    blocking::execute(move || {
        let csv = service.export_csv(period, chrono::Local::now().date_naive())?;
        std::fs::write(&path, csv).map_err(|error| {
            tracing::error!(%error, path, "export des analyses impossible");
            AppError::Validation("Le fichier n'a pas pu être écrit à l'emplacement choisi.".into())
        })
    })
    .await
}
