//! Frontière IPC des CV et lettres.

use crate::app::state::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::utils::blocking;
use crate::core::utils::validation::validate_user_file_path;
use crate::features::ai::domain::GeneratedResume;
use crate::features::documents::application::{build, build_cover_letter};
use crate::features::documents::domain::{
    CoverLetter, CoverLetterExport, NewCoverLetter, NewResume, ResumeSummary, ResumeVersion,
};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[tauri::command(rename_all = "snake_case")]
pub async fn documents_resume_list(state: State<'_, AppState>) -> AppResult<Vec<ResumeSummary>> {
    let service = state.documents.clone();
    blocking::execute(move || service.resume_list()).await
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
    state: State<'_, AppState>,
    resume: GeneratedResume,
    path: String,
) -> AppResult<()> {
    let profile = Arc::clone(&state.profile);
    blocking::execute(move || {
        let cible = validate_user_file_path(&path)?;
        let payload = profile.load()?;
        build(&payload.profile, &resume)
            .render_pdf(&cible)
            .map_err(|error| {
                tracing::error!(%error, path = %cible.display(), "export PDF impossible");
                AppError::Validation("Le PDF n'a pas pu être écrit à l'emplacement choisi.".into())
            })
    })
    .await
}

/// Exporte une lettre au chemin choisi dans le sélecteur natif.
///
/// L'identité du profil (nom, ville, e-mail) est posée en en-tête, comme
/// sur l'aperçu HTML.
#[tauri::command(rename_all = "snake_case")]
pub async fn documents_cover_letter_export_pdf(
    state: State<'_, AppState>,
    cover_letter: CoverLetterExport,
    path: String,
) -> AppResult<()> {
    let profile = Arc::clone(&state.profile);
    blocking::execute(move || {
        let cible = validate_user_file_path(&path)?;
        let payload = profile.load()?;
        build_cover_letter(&payload.profile, &cover_letter)
            .render_pdf(&cible)
            .map_err(|error| {
                tracing::error!(%error, path = %cible.display(), "export PDF de lettre impossible");
                AppError::Validation("Le PDF n'a pas pu être écrit à l'emplacement choisi.".into())
            })
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
