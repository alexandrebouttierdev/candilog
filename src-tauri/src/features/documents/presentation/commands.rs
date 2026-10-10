//! Frontière IPC des CV et lettres.

use crate::app::state::AppState;
use crate::core::clipboard;
use crate::core::errors::AppResult;
use crate::core::files::select_save_target;
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::ai::domain::{ProfileSection, ResumeGeneration};
use crate::features::documents::domain::{
    CoverLetter, CoverLetterExport, DocumentVersion, NewCoverLetter, NewResume, ResumeDocument,
    ResumeSummary, ResumeVersion, ResumeWorkspace,
};
use tauri::{AppHandle, State};
use uuid::Uuid;

/// Texte du presse-papiers, pour le bouton « Coller » des champs d'offre.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_read_clipboard(app: AppHandle) -> AppResult<String> {
    clipboard::read_text(&app)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_list_page(
    state: State<'_, AppState>,
    page: u64,
    page_size: u64,
    search: String,
    scored_only: Option<bool>,
) -> AppResult<Page<ResumeSummary>> {
    let service = state.documents.clone();
    blocking::execute(move || {
        service.resume_list_page(page, page_size, &search, scored_only.unwrap_or(false))
    })
    .await
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

/// Versions du CV auquel appartient `id`, de la plus récente à la plus ancienne.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_versions(
    state: State<'_, AppState>,
    id: Uuid,
) -> AppResult<Vec<DocumentVersion>> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_versions(id)).await
}

/// Fait d'une version la version courante de son CV.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_restore(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_restore(id)).await
}

/// Fige le profil et une génération IA dans un document de travail autonome, propositions
/// d'amélioration comprises.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_prepare(
    state: State<'_, AppState>,
    generation: ResumeGeneration,
    excluded_sections: Option<Vec<ProfileSection>>,
) -> AppResult<ResumeWorkspace> {
    let service = state.documents.clone();
    let excluded = excluded_sections.unwrap_or_default();
    blocking::execute(move || service.resume_prepare(generation, &excluded)).await
}

/// Compose un CV de base depuis le seul profil enregistré, sans offre ni IA.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_compose_base(
    state: State<'_, AppState>,
    excluded_sections: Vec<ProfileSection>,
) -> AppResult<ResumeDocument> {
    let service = state.documents.clone();
    blocking::execute(move || service.compose_base_resume(excluded_sections)).await
}

/// Revalide le document puis recalcule score et propositions après une édition manuelle.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_recalculate(
    state: State<'_, AppState>,
    workspace: ResumeWorkspace,
) -> AppResult<ResumeWorkspace> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_recalculate(workspace)).await
}

/// Applique une proposition puis recalcule le poste de travail.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_apply_proposal(
    state: State<'_, AppState>,
    workspace: ResumeWorkspace,
    proposal_id: String,
) -> AppResult<ResumeWorkspace> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_apply_proposal(workspace, &proposal_id)).await
}

/// Refuse une proposition sans modifier le document, puis recalcule le poste de travail.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_reject_proposal(
    state: State<'_, AppState>,
    workspace: ResumeWorkspace,
    proposal_id: String,
) -> AppResult<ResumeWorkspace> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_reject_proposal(workspace, &proposal_id)).await
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
    let service = state.documents.clone();
    blocking::execute(move || {
        service.resume_export_pdf(&document, &cible)?;
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
    let service = state.documents.clone();
    blocking::execute(move || {
        service.cover_letter_export_pdf(&cover_letter, &cible)?;
        Ok(true)
    })
    .await
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

/// Versions de la lettre à laquelle appartient `id`, de la plus récente à la plus ancienne.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_cover_letter_versions(
    state: State<'_, AppState>,
    id: Uuid,
) -> AppResult<Vec<DocumentVersion>> {
    let service = state.documents.clone();
    blocking::execute(move || service.cover_letter_versions(id)).await
}

/// Fait d'une version la version courante de sa lettre.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_cover_letter_restore(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    let service = state.documents.clone();
    blocking::execute(move || service.cover_letter_restore(id)).await
}
