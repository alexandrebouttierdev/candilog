//! Commands Tauri des candidatures.

use crate::app::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::files::{atomic_write, select_save_target};
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::applications::application::export;
use crate::features::applications::domain::{
    Application, ApplicationFilter, ApplicationStatus, NewApplication, PipelineBreakdown,
};
use std::sync::Arc;
use tauri::{AppHandle, State};

/// Payload une page de candidatures, filtrée et triée.
#[tauri::command(rename_all = "snake_case")]
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
#[tauri::command(rename_all = "snake_case")]
pub async fn applications_breakdown(
    state: State<'_, AppState>,
    filter: ApplicationFilter,
) -> AppResult<PipelineBreakdown> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.breakdown(&filter)).await
}

/// Récupère une candidature par identifiant.
#[tauri::command(rename_all = "snake_case")]
pub async fn applications_get(
    state: State<'_, AppState>,
    id: uuid::Uuid,
) -> AppResult<Application> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.get(id)).await
}

/// Crée une candidature.
#[tauri::command(rename_all = "snake_case")]
pub async fn applications_create(
    state: State<'_, AppState>,
    input: NewApplication,
) -> AppResult<Application> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.create(&input)).await
}

/// Remplace les champs d'une candidature.
#[tauri::command(rename_all = "snake_case")]
pub async fn applications_update(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    input: NewApplication,
) -> AppResult<Application> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.update(id, &input)).await
}

/// Change le seul statut — geste du glisser-déposer du Kanban.
#[tauri::command(rename_all = "snake_case")]
pub async fn applications_change_status(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    status: ApplicationStatus,
) -> AppResult<Application> {
    let service = Arc::clone(&state.applications);
    blocking::execute(move || service.change_status(id, status)).await
}

/// Supprime une candidature et, en cascade, ses relances, entretiens et historique.
#[tauri::command(rename_all = "snake_case")]
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
#[tauri::command(rename_all = "snake_case")]
pub async fn applications_export_csv(
    app: AppHandle,
    state: State<'_, AppState>,
    filter: ApplicationFilter,
) -> AppResult<Option<u64>> {
    let Some(cible) = select_save_target(
        &app,
        "Exporter les candidatures",
        "candidatures.csv",
        "Fichier CSV",
        "csv",
    )?
    else {
        return Ok(None);
    };
    let service = Arc::clone(&state.applications);
    blocking::execute(move || {
        let items = service.list_matching(&filter)?;
        let csv = export::vers_csv(&items)?;
        atomic_write(&cible, "csv", |temporaire| {
            std::fs::write(temporaire, &csv).map_err(|error| {
                tracing::error!(%error, "export CSV impossible");
                AppError::Validation(
                    "Le fichier n'a pas pu être écrit à l'emplacement choisi.".into(),
                )
            })
        })?;
        Ok(Some(items.len() as u64))
    })
    .await
}
