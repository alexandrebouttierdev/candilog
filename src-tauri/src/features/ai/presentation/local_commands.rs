//! Frontière IPC mince du fournisseur Mistral Local.

use crate::app::state::AppState;
use crate::core::errors::AppResult;
use crate::features::ai::domain::{
    InstallLocalAiRequest, LocalAiBenchmark, LocalAiDownloadCompleted, LocalAiDownloadError,
    LocalAiHardware, LocalAiRecommendation, LocalAiStatus, LocalModelId,
};
use tauri::{AppHandle, Emitter, State};

pub const DOWNLOAD_PROGRESS_EVENT: &str = "local-ai://download-progress";
pub const DOWNLOAD_COMPLETED_EVENT: &str = "local-ai://download-completed";
pub const DOWNLOAD_ERROR_EVENT: &str = "local-ai://download-error";

#[tauri::command(rename_all = "snake_case")]
pub fn detect_local_ai_hardware(state: State<'_, AppState>) -> LocalAiHardware {
    state.local_ai.detect_hardware()
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_local_ai_recommendation(state: State<'_, AppState>) -> LocalAiRecommendation {
    state.local_ai.recommendation()
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_local_ai_status(state: State<'_, AppState>) -> AppResult<LocalAiStatus> {
    state.local_ai.status()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn install_local_ai_model(
    app: AppHandle,
    state: State<'_, AppState>,
    request: InstallLocalAiRequest,
) -> AppResult<LocalAiStatus> {
    let model_id = request.model_id;
    let result = state
        .local_ai
        .install(request, |progress| {
            if let Err(error) = app.emit(DOWNLOAD_PROGRESS_EVENT, progress) {
                tracing::warn!(%error, "progression du modèle local non émise");
            }
        })
        .await;

    match &result {
        Ok(_) => {
            if let Err(error) = app.emit(
                DOWNLOAD_COMPLETED_EVENT,
                LocalAiDownloadCompleted { model_id },
            ) {
                tracing::warn!(%error, "fin d'installation du modèle local non émise");
            }
        }
        Err(error) => {
            let payload = LocalAiDownloadError {
                model_id,
                code: error.code().into(),
                message: error.user_message(),
            };
            if let Err(emit_error) = app.emit(DOWNLOAD_ERROR_EVENT, payload) {
                tracing::warn!(%emit_error, "erreur d'installation du modèle local non émise");
            }
        }
    }
    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn cancel_local_ai_download(state: State<'_, AppState>) {
    state.local_ai.cancel_download();
}

#[tauri::command(rename_all = "snake_case")]
pub async fn remove_local_ai_model(
    state: State<'_, AppState>,
    model_id: LocalModelId,
) -> AppResult<LocalAiStatus> {
    state.local_ai.remove(model_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn benchmark_local_ai_model(state: State<'_, AppState>) -> AppResult<LocalAiBenchmark> {
    state.local_ai.benchmark().await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn test_local_ai_model(state: State<'_, AppState>) -> AppResult<String> {
    state.local_ai.test_model().await
}
