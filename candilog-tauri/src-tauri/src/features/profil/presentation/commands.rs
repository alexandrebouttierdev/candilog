//! Commandes Tauri du profil professionnel.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::profil::domain::{Profil, ProfilCharge};
use std::sync::Arc;
use tauri::State;

/// Charge le profil et son état de complétion.
#[tauri::command]
pub async fn profil_charger(state: State<'_, AppState>) -> AppResult<ProfilCharge> {
    let service = Arc::clone(&state.profil);
    blocking::execute(move || service.charger()).await
}

/// Remplace le profil complet après validation.
#[tauri::command]
pub async fn profil_enregistrer(
    state: State<'_, AppState>,
    profil: Profil,
) -> AppResult<ProfilCharge> {
    let service = Arc::clone(&state.profil);
    blocking::execute(move || service.enregistrer(&profil)).await
}
