//! Frontière IPC des CV et lettres.

use crate::app::state::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::utils::blocking;
use crate::features::documents::application::{construire, construire_lettre};
use crate::features::documents::domain::{
    CvResume, CvVersion, ExportLettre, Lettre, NouveauCv, NouvelleLettre,
};
use crate::features::ia::domain::CvGenere;
use std::path::Path;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn documents_cv_lister(state: State<'_, AppState>) -> AppResult<Vec<CvResume>> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.cv_lister())
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_cv_obtenir(state: State<'_, AppState>, id: Uuid) -> AppResult<CvVersion> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.cv_obtenir(id))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_cv_enregistrer(
    state: State<'_, AppState>,
    input: NouveauCv,
) -> AppResult<CvVersion> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.cv_enregistrer(&input))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_cv_supprimer(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.cv_supprimer(id))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}

/// Exporte un CV généré au chemin choisi dans le sélecteur natif.
///
/// Le profil (identité, périodes, projets, langues) est fusionné au contenu
/// reformulé : l'aperçu HTML et le PDF reposent sur les mêmes données.
#[tauri::command]
pub async fn documents_cv_exporter_pdf(
    state: State<'_, AppState>,
    cv: CvGenere,
    chemin: String,
) -> AppResult<()> {
    let profil = Arc::clone(&state.profil);
    blocking::execute(move || {
        let charge = profil.charger()?;
        construire(&charge.profil, &cv)
            .render_pdf(Path::new(&chemin))
            .map_err(|error| {
                tracing::error!(%error, chemin, "export PDF impossible");
                AppError::Validation("Le PDF n'a pas pu être écrit à l'emplacement choisi.".into())
            })
    })
    .await
}

/// Exporte une lettre au chemin choisi dans le sélecteur natif.
///
/// L'identité du profil (nom, ville, e-mail) est posée en en-tête, comme
/// sur l'aperçu HTML.
#[tauri::command]
pub async fn documents_lettre_exporter_pdf(
    state: State<'_, AppState>,
    lettre: ExportLettre,
    chemin: String,
) -> AppResult<()> {
    let profil = Arc::clone(&state.profil);
    blocking::execute(move || {
        let charge = profil.charger()?;
        construire_lettre(&charge.profil, &lettre)
            .render_pdf(Path::new(&chemin))
            .map_err(|error| {
                tracing::error!(%error, chemin, "export PDF de lettre impossible");
                AppError::Validation("Le PDF n'a pas pu être écrit à l'emplacement choisi.".into())
            })
    })
    .await
}
#[tauri::command]
pub async fn documents_lettres_lister(state: State<'_, AppState>) -> AppResult<Vec<Lettre>> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.lettres_lister())
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_lettre_obtenir(state: State<'_, AppState>, id: Uuid) -> AppResult<Lettre> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.lettre_obtenir(id))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_lettre_enregistrer(
    state: State<'_, AppState>,
    input: NouvelleLettre,
) -> AppResult<Lettre> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.lettre_enregistrer(&input))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
#[tauri::command]
pub async fn documents_lettre_supprimer(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    let service = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || service.lettre_supprimer(id))
        .await
        .map_err(|e| crate::core::errors::AppError::Database(e.to_string()))?
}
