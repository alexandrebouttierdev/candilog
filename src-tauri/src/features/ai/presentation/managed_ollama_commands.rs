//! Frontière IPC du runtime Ollama géré par Candilog.

use crate::app::state::AppState;
use crate::core::errors::AppResult;
use crate::features::ai::domain::{
    InstallManagedModelRequest, ManagedModelDefinition, ManagedModelId, ManagedOllamaStatus,
    UserBenchmarkResult,
};
use tauri::{AppHandle, Emitter, State};

pub const MANAGED_DOWNLOAD_PROGRESS_EVENT: &str = "managed-ollama://download-progress";

#[tauri::command(rename_all = "snake_case")]
pub fn get_managed_ollama_status(state: State<'_, AppState>) -> AppResult<ManagedOllamaStatus> {
    state.managed_ollama.status()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn install_managed_ollama_model(
    app: AppHandle,
    state: State<'_, AppState>,
    request: InstallManagedModelRequest,
) -> AppResult<ManagedOllamaStatus> {
    state
        .managed_ollama
        .install_model(request, |progress| {
            if let Err(error) = app.emit(MANAGED_DOWNLOAD_PROGRESS_EVENT, progress) {
                tracing::warn!(%error, "progression Ollama géré non émise");
            }
        })
        .await?;
    state.managed_ollama.status()
}

#[tauri::command(rename_all = "snake_case")]
pub fn cancel_managed_ollama_download(state: State<'_, AppState>) -> AppResult<()> {
    state.managed_ollama.cancel_download()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn remove_managed_ollama_model(
    state: State<'_, AppState>,
    model_id: ManagedModelId,
) -> AppResult<ManagedOllamaStatus> {
    state.managed_ollama.remove_model(model_id).await?;
    state.managed_ollama.status()
}

#[tauri::command(rename_all = "snake_case")]
pub fn activate_managed_ollama_model(
    state: State<'_, AppState>,
    model_id: ManagedModelId,
) -> AppResult<ManagedModelDefinition> {
    state.managed_ollama.activate_model(model_id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn run_user_cv_benchmark(
    state: State<'_, AppState>,
    generation_id: String,
) -> AppResult<UserBenchmarkResult> {
    state.ai.run_user_cv_benchmark(generation_id).await
}
