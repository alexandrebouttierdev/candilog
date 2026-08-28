//! Commands Tauri des candidatures.

use crate::app::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::applications::application::export;
use crate::features::applications::domain::{
    Application, ApplicationFilter, NewApplication, PipelineBreakdown, ApplicationStatus,
};
use std::sync::Arc;
use tauri::State;

/// Payload une page de candidatures, filtrée et triée.
#[tauri::command]
pub async fn applications_list_page(
    state: State<'_, AppState>,
    page: u64,
    page_size: u64,
    filter: ApplicationFilter,
) -> AppResult<Page<Application>> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.list_page(page, page_size, &filter)).await
}

/// Report les candidatures par statut, pour les en-têtes de colonnes du Kanban.
#[tauri::command]
pub async fn applications_breakdown(
    state: State<'_, AppState>,
    filter: ApplicationFilter,
) -> AppResult<PipelineBreakdown> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.breakdown(&filter)).await
}

/// Récupère une candidature par identifiant.
#[tauri::command]
pub async fn applications_get(
    state: State<'_, AppState>,
    id: uuid::Uuid,
) -> AppResult<Application> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.get(id)).await
}

/// Crée une candidature.
#[tauri::command]
pub async fn applications_create(
    state: State<'_, AppState>,
    input: NewApplication,
) -> AppResult<Application> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.create(&input)).await
}

/// Remplace les champs d'une candidature.
#[tauri::command]
pub async fn applications_update(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    input: NewApplication,
) -> AppResult<Application> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.update(id, &input)).await
}

/// Change le seul statut — geste du glisser-déposer du Kanban.
#[tauri::command]
pub async fn applications_change_status(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    status: ApplicationStatus,
) -> AppResult<Application> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.change_status(id, status)).await
}

/// Supprime une candidature et, en cascade, ses relances, entretiens et historique.
#[tauri::command]
pub async fn applications_delete(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.delete(id)).await
}

/// Exporte en CSV les candidatures correspondant au filtre courant, au chemin choisi.
///
/// Le chemin vient du sélecteur de fichiers natif, ouvert côté frontend par le plugin
/// `dialog` : l'utilisateur désigne lui-même la destination, et la commande n'écrit nulle
/// part ailleurs. Aucune permission filesystem large n'est accordée à la fenêtre (§44).
///
/// L'export porte sur **tout le filtre** et non sur la page affichée : exporter huit lignes
/// sur quarante serait un piège silencieux.
#[tauri::command]
pub async fn applications_export_csv(
    state: State<'_, AppState>,
    filter: ApplicationFilter,
    path: String,
) -> AppResult<u64> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || {
        let page = service.list_page(1, u64::MAX, &filter)?;
        let csv = export::vers_csv(&page.items)?;
        std::fs::write(&path, csv).map_err(|error| {
            tracing::error!(%error, path, "export CSV impossible");
            AppError::Validation("Le fichier n'a pas pu être écrit à l'emplacement choisi.".into())
        })?;
        Ok(page.total)
    })
    .await
}
