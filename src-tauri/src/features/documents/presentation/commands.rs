//! Frontière IPC des CV et lettres.

use crate::app::state::AppState;
use crate::core::errors::AppResult;
use crate::features::documents::domain::{CvResume, CvVersion, Lettre, NouveauCv, NouvelleLettre};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn documents_cv_lister(state: State<'_, AppState>) -> AppResult<Vec<CvResume>> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.cv_lister())
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_cv_obtenir(state: State<'_, AppState>, id: Uuid) -> AppResult<CvVersion> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.cv_obtenir(id))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_cv_enregistrer(
    state: State<'_, AppState>,
    input: NouveauCv,
) -> AppResult<CvVersion> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.cv_enregistrer(&input))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_cv_supprimer(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.cv_supprimer(id))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_lettres_lister(state: State<'_, AppState>) -> AppResult<Vec<Lettre>> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.lettres_lister())
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_lettre_obtenir(state: State<'_, AppState>, id: Uuid) -> AppResult<Lettre> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.lettre_obtenir(id))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_lettre_enregistrer(
    state: State<'_, AppState>,
    input: NouvelleLettre,
) -> AppResult<Lettre> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.lettre_enregistrer(&input))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_lettre_supprimer(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.lettre_supprimer(id))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
