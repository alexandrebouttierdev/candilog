//! Commandes Tauri du tableau de bord et des analyses.

use crate::app::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::utils::blocking;
use crate::features::analyses::domain::{Analyses, Periode, TableauDeBord};
use std::sync::Arc;
use tauri::State;

/// Charge le tableau de bord à la date locale courante.
#[tauri::command]
pub async fn analyses_tableau_de_bord(state: State<'_, AppState>) -> AppResult<TableauDeBord> {
    let service = Arc::clone(&state.analyses);
    blocking::execute(move || service.tableau_de_bord(chrono::Local::now().date_naive())).await
}

/// Charge l'écran Analyses pour la période choisie.
#[tauri::command]
pub async fn analyses_charger(state: State<'_, AppState>, periode: Periode) -> AppResult<Analyses> {
    let service = Arc::clone(&state.analyses);
    blocking::execute(move || service.analyses(periode, chrono::Local::now().date_naive())).await
}

/// Écrit l'export CSV des analyses au chemin choisi dans le sélecteur natif.
#[tauri::command]
pub async fn analyses_exporter_csv(
    state: State<'_, AppState>,
    periode: Periode,
    chemin: String,
) -> AppResult<()> {
    let service = Arc::clone(&state.analyses);
    blocking::execute(move || {
        let csv = service.exporter_csv(periode, chrono::Local::now().date_naive())?;
        std::fs::write(&chemin, csv).map_err(|error| {
            tracing::error!(%error, chemin, "export des analyses impossible");
            AppError::Validation("Le fichier n'a pas pu être écrit à l'emplacement choisi.".into())
        })
    })
    .await
}
