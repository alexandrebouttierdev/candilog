//! Commands Tauri du profil professionnel.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::profile::domain::{Profile, ProfilePayload};
use std::sync::Arc;
use tauri::State;

/// Payload le profil et son état de complétion.
#[tauri::command]
pub async fn profile_load(state: State<'_, AppState>) -> AppResult<ProfilePayload> {
    let service = Arc::clone(&state.profile);
    blocking::execute(move || service.load()).await
}

/// Remplace le profil complet après validation.
#[tauri::command]
pub async fn profile_save(
    state: State<'_, AppState>,
    profile: Profile,
) -> AppResult<ProfilePayload> {
    let service = Arc::clone(&state.profile);
    blocking::execute(move || service.save(&profile)).await
}
