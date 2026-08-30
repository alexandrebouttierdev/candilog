//! Commands Tauri du tableau de bord et des analyses.

use crate::app::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::files::{atomic_write, select_save_target};
use crate::core::utils::blocking;
use crate::features::analytics::domain::{Analytics, Dashboard, Period};
use std::sync::Arc;
use tauri::{AppHandle, State};

/// Payload le tableau de bord à la date locale courante.
#[tauri::command(rename_all = "snake_case")]
pub async fn analytics_dashboard(state: State<'_, AppState>) -> AppResult<Dashboard> {
    let service = Arc::clone(&state.analytics);
    blocking::execute(move || service.dashboard(chrono::Local::now().date_naive())).await
}

/// Payload l'écran Analytics pour la période choisie.
#[tauri::command(rename_all = "snake_case")]
pub async fn analytics_load(state: State<'_, AppState>, period: Period) -> AppResult<Analytics> {
    let service = Arc::clone(&state.analytics);
    blocking::execute(move || service.analytics(period, chrono::Local::now().date_naive())).await
}

/// Écrit l'export CSV des analyses au chemin choisi dans le sélecteur natif.
#[tauri::command(rename_all = "snake_case")]
pub async fn analytics_export_csv(
    app: AppHandle,
    state: State<'_, AppState>,
    period: Period,
) -> AppResult<bool> {
    let Some(cible) = select_save_target(
        &app,
        "Exporter les analyses",
        "analyses.csv",
        "Fichier CSV",
        "csv",
    )?
    else {
        return Ok(false);
    };
    let service = Arc::clone(&state.analytics);
    blocking::execute(move || {
        let csv = service.export_csv(period, chrono::Local::now().date_naive())?;
        atomic_write(&cible, "csv", |temporaire| {
            std::fs::write(temporaire, &csv).map_err(|error| {
                tracing::error!(%error, "export des analyses impossible");
                AppError::Validation(
                    "Le fichier n'a pas pu être écrit à l'emplacement choisi.".into(),
                )
            })
        })?;
        Ok(true)
    })
    .await
}
