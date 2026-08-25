//! Commandes Tauri des relances.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::relances::domain::{NouvelleRelance, Relance};
use std::sync::Arc;
use tauri::State;

/// Liste les relances d'une plage de dates, bornes incluses.
#[tauri::command]
pub async fn relances_lister_entre(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> AppResult<Vec<Relance>> {
    let service = Arc::clone(&state.relances);
    blocking::execute(move || service.lister_entre(&from, &to)).await
}

/// Crée une relance.
#[tauri::command]
pub async fn relances_creer(
    state: State<'_, AppState>,
    input: NouvelleRelance,
) -> AppResult<Relance> {
    let service = Arc::clone(&state.relances);
    blocking::execute(move || service.creer(&input)).await
}

/// Remplace les champs d'une relance.
#[tauri::command]
pub async fn relances_modifier(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    input: NouvelleRelance,
) -> AppResult<Relance> {
    let service = Arc::clone(&state.relances);
    blocking::execute(move || service.modifier(id, &input)).await
}

/// Supprime une relance.
#[tauri::command]
pub async fn relances_supprimer(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.relances);
    blocking::execute(move || service.supprimer(id)).await
}
