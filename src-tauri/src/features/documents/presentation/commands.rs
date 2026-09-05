//! Frontière IPC des CV et lettres.

use crate::app::state::AppState;
use crate::core::clipboard;
use crate::core::errors::{AppError, AppResult};
use crate::core::files::{atomic_write, select_save_target};
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::ai::domain::ResumeGeneration;
use crate::features::documents::application::{
    apply_proposal, build, build_cover_letter, prepare_workspace, recalculate, reject_proposal,
};
use crate::features::documents::domain::{
    CoverLetter, CoverLetterExport, NewCoverLetter, NewResume, ResumeDocument, ResumeSummary,
    ResumeVersion, ResumeWorkspace,
};
use std::sync::Arc;
use tauri::{AppHandle, State};
use uuid::Uuid;

/// Texte du presse-papiers, pour le bouton « Coller » des champs d'offre.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_read_clipboard(app: AppHandle) -> AppResult<String> {
    clipboard::read_text(&app)
}

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

/// Fige le profil et une génération IA dans un document de travail autonome, propositions
/// d'amélioration comprises.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_prepare(
    state: State<'_, AppState>,
    generation: ResumeGeneration,
) -> AppResult<ResumeWorkspace> {
    let profile = Arc::clone(&state.profile);
    blocking::execute(move || {
        let payload = profile.load()?;
        let photo = profile
            .photo_path()?
            .and_then(|path| std::fs::read(path).ok());
        prepare_workspace(&payload.profile, generation, photo)
    })
    .await
}

/// Revalide le document puis recalcule score et propositions après une édition manuelle.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_recalculate(
    state: State<'_, AppState>,
    workspace: ResumeWorkspace,
) -> AppResult<ResumeWorkspace> {
    let profile = Arc::clone(&state.profile);
    blocking::execute(move || {
        let photo = profile
            .photo_path()?
            .and_then(|path| std::fs::read(path).ok());
        recalculate(workspace, photo)
    })
    .await
}

/// Applique une proposition puis recalcule le poste de travail.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_apply_proposal(
    state: State<'_, AppState>,
    workspace: ResumeWorkspace,
    proposal_id: String,
) -> AppResult<ResumeWorkspace> {
    let profile = Arc::clone(&state.profile);
    blocking::execute(move || {
        let photo = profile
            .photo_path()?
            .and_then(|path| std::fs::read(path).ok());
        apply_proposal(workspace, &proposal_id, photo)
    })
    .await
}

/// Refuse une proposition sans modifier le document, puis recalcule le poste de travail.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_reject_proposal(
    state: State<'_, AppState>,
    workspace: ResumeWorkspace,
    proposal_id: String,
) -> AppResult<ResumeWorkspace> {
    let profile = Arc::clone(&state.profile);
    blocking::execute(move || {
        let photo = profile
            .photo_path()?
            .and_then(|path| std::fs::read(path).ok());
        reject_proposal(workspace, &proposal_id, photo)
    })
    .await
}

/// Exporte un document CV autonome au chemin choisi dans le sélecteur natif.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_export_pdf(
    app: AppHandle,
    state: State<'_, AppState>,
    document: ResumeDocument,
) -> AppResult<bool> {
    let Some(cible) = select_save_target(&app, "Exporter le CV", "cv.pdf", "Document PDF", "pdf")?
    else {
        return Ok(false);
    };
    let profile = std::sync::Arc::clone(&state.profile);
    blocking::execute(move || {
        // La photo suit le profil courant, pas la version de CV enregistrée : un CV rouvert
        // après suppression de la photo s'exporte sans elle, sans laisser de cadre vide.
        let photo = profile
            .photo_path()?
            .and_then(|chemin| std::fs::read(chemin).ok());
        let bytes = build(&document, photo).render_bytes()?;
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
