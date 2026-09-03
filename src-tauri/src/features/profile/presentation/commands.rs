//! Commands Tauri du profil professionnel.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::files::select_source;
use crate::core::utils::blocking;
use crate::features::profile::domain::{
    ImportProfileRequest, ImportProfileResult, Profile, ProfilePayload, ACCEPTED_EXTENSIONS,
};
use std::sync::Arc;
use tauri::State;

/// Payload le profil et son état de complétion.
#[tauri::command(rename_all = "snake_case")]
pub async fn profile_load(state: State<'_, AppState>) -> AppResult<ProfilePayload> {
    let service = Arc::clone(&state.profile);
    blocking::execute(move || service.load()).await
}

/// Remplace le profil complet après validation.
#[tauri::command(rename_all = "snake_case")]
pub async fn profile_save(
    state: State<'_, AppState>,
    profile: Profile,
) -> AppResult<ProfilePayload> {
    let service = Arc::clone(&state.profile);
    blocking::execute(move || service.save(&profile)).await
}

/// Applique un import de CV après revue utilisateur.
#[tauri::command(rename_all = "snake_case")]
pub async fn profile_apply_import(
    state: State<'_, AppState>,
    request: ImportProfileRequest,
) -> AppResult<ImportProfileResult> {
    let service = Arc::clone(&state.profile);
    blocking::execute(move || service.apply_import(&request)).await
}

/// Ajoute une compétence au profil, sans doublon, depuis une proposition de l'éditeur de CV.
#[tauri::command(rename_all = "snake_case")]
pub async fn profile_add_skill(
    state: State<'_, AppState>,
    name: String,
) -> AppResult<ProfilePayload> {
    let service = Arc::clone(&state.profile);
    blocking::execute(move || service.add_skill(&name)).await
}

/// Ouvre le sélecteur de fichier natif et remplace la photo du profil.
///
/// Retourne `None` si l'utilisateur annule : une annulation n'est pas une erreur.
#[tauri::command(rename_all = "snake_case")]
pub async fn profile_set_photo(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Option<ProfilePayload>> {
    let Some(source) = select_source(&app, "Choisir une photo", "Image", ACCEPTED_EXTENSIONS)?
    else {
        return Ok(None);
    };
    let service = Arc::clone(&state.profile);
    blocking::execute(move || service.set_photo(&source).map(Some)).await
}

/// Retire la photo du profil et supprime son fichier.
#[tauri::command(rename_all = "snake_case")]
pub async fn profile_remove_photo(state: State<'_, AppState>) -> AppResult<ProfilePayload> {
    let service = Arc::clone(&state.profile);
    blocking::execute(move || service.remove_photo()).await
}

/// Photo du profil encodée en `data:` URL, ou `None` sans photo.
#[tauri::command(rename_all = "snake_case")]
pub async fn profile_photo(state: State<'_, AppState>) -> AppResult<Option<String>> {
    let service = Arc::clone(&state.profile);
    blocking::execute(move || service.photo_data_url()).await
}

/// Réinitialise le seul profil professionnel, photo comprise.
///
/// Aucune autre donnée n'est touchée : la commande n'atteint que la ligne `profile`.
#[tauri::command(rename_all = "snake_case")]
pub async fn profile_reset(state: State<'_, AppState>) -> AppResult<ProfilePayload> {
    let service = Arc::clone(&state.profile);
    blocking::execute(move || service.reset()).await
}
