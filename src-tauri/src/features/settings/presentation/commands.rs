//! Frontière IPC des réglages, sauvegardes et mises à jour.

use crate::app::state::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::settings::domain::{
    About, LlmForm, UpdateInfo, Settings, UpdateProgress,
};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// Payload les réglages, clé API comprise (coffre).
#[tauri::command]
pub async fn settings_load(state: State<'_, AppState>) -> AppResult<Settings> {
    let service = Arc::clone(&state.settings);
    blocking::execute(move || service.load()).await
}

/// Valide et persiste les réglages. La clé API quitte SQLite pour le coffre.
#[tauri::command]
pub async fn settings_save(
    state: State<'_, AppState>,
    settings: Settings,
) -> AppResult<Settings> {
    let service = Arc::clone(&state.settings);
    blocking::execute(move || service.save(settings)).await
}

/// Teste le fournisseur décrit par le formulaire, sans l'enregistrer.
#[tauri::command]
pub async fn settings_test_connection(
    state: State<'_, AppState>,
    llm: LlmForm,
) -> AppResult<()> {
    state.settings.test_connection(llm).await
}

/// List les modèles du fournisseur décrit par le formulaire.
#[tauri::command]
pub async fn settings_list_models(
    state: State<'_, AppState>,
    llm: LlmForm,
) -> AppResult<Vec<String>> {
    state.settings.list_models(llm).await
}

/// Vide le cache des réponses IA.
#[tauri::command]
pub async fn parametres_clear_ai_cache(state: State<'_, AppState>) -> AppResult<()> {
    let service = Arc::clone(&state.settings);
    blocking::execute(move || service.clear_ai_cache()).await
}

/// Exporte la base vers le chemin choisi dans le sélecteur natif.
#[tauri::command]
pub async fn settings_export(state: State<'_, AppState>, path: String) -> AppResult<()> {
    let service = Arc::clone(&state.settings);
    blocking::execute(move || service.export(PathBuf::from(path).as_path())).await
}

/// Restaure un backup validé, avec retour arrière en cas d'échec.
#[tauri::command]
pub async fn settings_restore(state: State<'_, AppState>, path: String) -> AppResult<()> {
    let service = Arc::clone(&state.settings);
    blocking::execute(move || service.restore(PathBuf::from(path).as_path())).await
}

/// Efface les données utilisateur, pas le référentiel des secteurs.
#[tauri::command]
pub async fn settings_reset(state: State<'_, AppState>) -> AppResult<()> {
    let service = Arc::clone(&state.settings);
    blocking::execute(move || service.reset()).await
}

/// Compare la version installée à la dernière release GitHub.
#[tauri::command]
pub async fn settings_check_update(state: State<'_, AppState>) -> AppResult<Option<UpdateInfo>> {
    state.settings.check_update().await
}

/// Télécharge l'installeur (événement `maj-progression`) puis l'ouvre.
#[tauri::command]
pub async fn settings_download_update(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
    name: String,
) -> AppResult<String> {
    let notifier = move |progress: u8| {
        if let Err(error) = app.emit("maj-progression", UpdateProgress { progress }) {
            tracing::warn!(%error, "progression de mise à jour non émise");
        }
    };
    let path = state.settings.download_update(url, name, notifier).await?;
    Ok(path.to_string_lossy().into_owned())
}

/// Version et nom affichés sur l'écran À propos.
#[tauri::command]
pub async fn settings_about(state: State<'_, AppState>) -> AppResult<About> {
    Ok(state.settings.about())
}
