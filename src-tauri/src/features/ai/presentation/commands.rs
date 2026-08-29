//! Frontière IPC avec événements globaux `ia-progression`.

use crate::app::state::AppState;
use crate::core::errors::AppResult;
use crate::features::ai::domain::{
    AiProgress, CoverLetterRequest, ImportedResumeAnalysis, ListingAnalysis, ProfileImportProgress,
    ProfileImportRequest, ResumeAnalysisRequest, ResumeGeneration, ResumeGenerationRequest,
};
use crate::features::profile::domain::ImportProfilePreview;
use tauri::{AppHandle, Emitter, State};

fn notifier(app: AppHandle) -> impl Fn(AiProgress) {
    move |progress| {
        if let Err(error) = app.emit("ia-progression", progress) {
            tracing::warn!(%error, "progression IA non émise");
        }
    }
}

fn import_notifier(app: AppHandle) -> impl Fn(ProfileImportProgress) {
    move |progress| {
        if let Err(error) = app.emit("profile_import_progress", progress) {
            tracing::warn!(%error, "journal d'import non émis");
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_analyze_listing(
    state: State<'_, AppState>,
    text: String,
) -> AppResult<ListingAnalysis> {
    state.ai.analyze_listing(text).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_generate_resume(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ResumeGenerationRequest,
) -> AppResult<ResumeGeneration> {
    state.ai.generate_resume(request, notifier(app)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_generate_cover_letter(
    app: AppHandle,
    state: State<'_, AppState>,
    request: CoverLetterRequest,
) -> AppResult<String> {
    state.ai.generate_cover_letter(request, notifier(app)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_analyze_resume(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ResumeAnalysisRequest,
) -> AppResult<ImportedResumeAnalysis> {
    state
        .ai
        .analyze_resume_imported(request, notifier(app))
        .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_import_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ProfileImportRequest,
) -> AppResult<ImportProfilePreview> {
    state.ai.import_profile(request, import_notifier(app)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_cancel(state: State<'_, AppState>, generation_id: String) -> AppResult<()> {
    state.ai.cancel(&generation_id);
    Ok(())
}
