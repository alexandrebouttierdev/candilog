//! Frontière IPC des CV et lettres.

use crate::app::state::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::files::{atomic_write, select_save_target};
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::ai::domain::GeneratedResume;
use crate::features::documents::application::{build, build_cover_letter};
use crate::features::documents::domain::{
    CoverLetter, CoverLetterExport, NewCoverLetter, NewResume, ResumeSummary, ResumeVersion,
};
use std::sync::Arc;
use tauri::{AppHandle, State};
use uuid::Uuid;

#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_list(state: State<'_, AppState>) -> AppResult<Vec<ResumeSummary>> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_list()).await
}
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_list_page(
    state: State<'_, AppState>,
    page: u64,
    page_size: u64,
    search: String,
) -> AppResult<Page<ResumeSummary>> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_list_page(page, page_size, &search)).await
}
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> AppResult<ResumeVersion> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_get(id)).await
}
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_save(
    state: State<'_, AppState>,
    input: NewResume,
) -> AppResult<ResumeVersion> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_save(&input)).await
}
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_delete(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_delete(id)).await
}

/// Exporte un CV généré au chemin choisi dans le sélecteur natif.
///
/// Le profil (identité, périodes, projets, langues) est fusionné au contenu
/// reformulé : l'aperçu HTML et le PDF reposent sur les mêmes données.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_export_pdf(
    app: AppHandle,
    state: State<'_, AppState>,
    resume: GeneratedResume,
) -> AppResult<bool> {
    let Some(cible) = select_save_target(&app, "Exporter le CV", "cv.pdf", "Document PDF", "pdf")?
    else {
        return Ok(false);
    };
    let profile = Arc::clone(&state.profile);
    blocking::execute(move || {
        let payload = profile.load()?;
        let bytes = build(&payload.profile, &resume).render_bytes()?;
        atomic_write(&cible, "pdf", |temporaire| {
            std::fs::write(temporaire, &bytes).map_err(|error| {
                tracing::error!(%error, "export PDF impossible");
                AppError::Database(format!("Écriture du PDF impossible : {error}"))
            })
        })?;
        Ok(true)
    })
    .await
}

/// Exporte une lettre au chemin choisi dans le sélecteur natif.
///
/// L'identité du profil (nom, ville, e-mail) est posée en en-tête, comme
/// sur l'aperçu HTML.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_cover_letter_export_pdf(
    app: AppHandle,
    state: State<'_, AppState>,
    cover_letter: CoverLetterExport,
) -> AppResult<bool> {
    let Some(cible) = select_save_target(
        &app,
        "Exporter la lettre de motivation",
        "lettre-de-motivation.pdf",
        "Document PDF",
        "pdf",
    )?
    else {
        return Ok(false);
    };
    let profile = Arc::clone(&state.profile);
    blocking::execute(move || {
        let payload = profile.load()?;
        let bytes = build_cover_letter(&payload.profile, &cover_letter).render_bytes()?;
        atomic_write(&cible, "pdf", |temporaire| {
            std::fs::write(temporaire, &bytes).map_err(|error| {
                tracing::error!(%error, "export PDF de lettre impossible");
                AppError::Database(format!("Écriture du PDF de lettre impossible : {error}"))
            })
        })?;
        Ok(true)
    })
    .await
}
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_cover_letters_list(
    state: State<'_, AppState>,
) -> AppResult<Vec<CoverLetter>> {
    let service = state.documents.clone();
    blocking::execute(move || service.cover_letters_list()).await
}
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_cover_letters_list_page(
    state: State<'_, AppState>,
    page: u64,
    page_size: u64,
    search: String,
) -> AppResult<Page<CoverLetter>> {
    let service = state.documents.clone();
    blocking::execute(move || service.cover_letters_list_page(page, page_size, &search)).await
}
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_cover_letter_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> AppResult<CoverLetter> {
    let service = state.documents.clone();
    blocking::execute(move || service.cover_letter_get(id)).await
}
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_cover_letter_save(
    state: State<'_, AppState>,
    input: NewCoverLetter,
) -> AppResult<CoverLetter> {
    let service = state.documents.clone();
    blocking::execute(move || service.cover_letter_save(&input)).await
}
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_cover_letter_delete(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    let service = state.documents.clone();
    blocking::execute(move || service.cover_letter_delete(id)).await
}
