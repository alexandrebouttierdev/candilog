//! Frontière IPC avec événements globaux `ia-progression`.

use crate::app::state::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::files::select_source;
use crate::features::ai::domain::{
    AiExecution, AiProgress, CoverLetterRequest, ImportedResumeAnalysis, ListingAnalysis,
    ProfileImportProgress, ProfileImportRequest, ResumeAnalysisRequest, ResumeGeneration,
    ResumeGenerationRequest, SelectedResumeFile,
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
) -> AppResult<AiExecution<ListingAnalysis>> {
    state.ai.analyze_listing(text).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_generate_resume(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ResumeGenerationRequest,
) -> AppResult<AiExecution<ResumeGeneration>> {
    state.ai.generate_resume(request, notifier(app)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_generate_cover_letter(
    app: AppHandle,
    state: State<'_, AppState>,
    request: CoverLetterRequest,
) -> AppResult<AiExecution<String>> {
    state.ai.generate_cover_letter(request, notifier(app)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_analyze_resume(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ResumeAnalysisRequest,
) -> AppResult<AiExecution<ImportedResumeAnalysis>> {
    state
        .ai
        .analyze_resume_imported(request, notifier(app))
        .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_select_resume_file(app: AppHandle) -> AppResult<Option<SelectedResumeFile>> {
    let Some(path) = select_source(&app, "Choisir un CV", "Document PDF", &["pdf"])? else {
        return Ok(None);
    };
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            AppError::Validation("Le nom du fichier sélectionné est invalide.".into())
        })?;
    let path_text = path
        .to_str()
        .ok_or_else(|| AppError::Validation("Le chemin sélectionné est invalide.".into()))?;
    Ok(Some(SelectedResumeFile {
        path: path_text.to_owned(),
        name: name.to_owned(),
    }))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_import_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ProfileImportRequest,
) -> AppResult<Option<AiExecution<ImportProfilePreview>>> {
    let Some(path) = select_source(&app, "Importer un CV", "Document PDF", &["pdf"])? else {
        return Ok(None);
    };
    state
        .ai
        .import_profile(request, path, import_notifier(app))
        .await
        .map(Some)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_cancel(state: State<'_, AppState>, generation_id: String) -> AppResult<()> {
    state.ai.cancel(&generation_id);
    Ok(())
}
