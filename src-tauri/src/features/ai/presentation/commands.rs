//! Frontière IPC avec événements globaux `ia-progression`.

use crate::app::state::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::files::select_source;
use crate::features::ai::domain::{
    ActiveModelCapabilities, AiExecution, AiProgress, CoverLetterRequest, ImportedResumeAnalysis,
    LanguageCorrectionRequest, LanguageCorrectionResult, LetterFit, LetterFitRequest,
    ListingAnalysis, ProfileImportAnalysis, ProfileImportProgress, ProfileImportRequest,
    ResumeAnalysisRequest, ResumeGeneration, ResumeGenerationRequest, SelectedResumeFile,
};
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

/// Adéquation d'une lettre à l'offre, calculée localement.
#[tauri::command(rename_all = "snake_case")]
pub async fn ai_evaluate_cover_letter(
    state: State<'_, AppState>,
    request: LetterFitRequest,
) -> AppResult<LetterFit> {
    state.ai.evaluate_cover_letter(request)
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
pub async fn ai_correct_french(
    app: AppHandle,
    state: State<'_, AppState>,
    request: LanguageCorrectionRequest,
) -> AppResult<AiExecution<LanguageCorrectionResult>> {
    state.ai.correct_french(request, notifier(app)).await
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

/// Ouvre le dialogue natif et retient le CV choisi ; seul son nom revient à l'écran.
#[tauri::command(rename_all = "snake_case")]
pub async fn ai_select_resume_file(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Option<SelectedResumeFile>> {
    let Some(path) = select_source(&app, "Choisir un CV", "Document PDF", &["pdf"])? else {
        return Ok(None);
    };
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AppError::Validation("Le nom du fichier sélectionné est invalide.".into()))?
        .to_owned();
    state.ai.remember_selected_resume(path);
    Ok(Some(SelectedResumeFile { name }))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_import_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ProfileImportRequest,
) -> AppResult<Option<AiExecution<ProfileImportAnalysis>>> {
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
pub async fn ai_active_model_capabilities(
    state: State<'_, AppState>,
) -> AppResult<ActiveModelCapabilities> {
    state.ai.active_model_capabilities().await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn ai_cancel(state: State<'_, AppState>, generation_id: String) -> AppResult<()> {
    state.ai.cancel(&generation_id);
    Ok(())
}
