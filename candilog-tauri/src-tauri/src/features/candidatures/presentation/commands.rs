//! Commandes Tauri des candidatures.

use crate::app::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::candidatures::application::export;
use crate::features::candidatures::domain::{
    Candidature, FiltreCandidatures, NouvelleCandidature, RepartitionPipeline, StatutCandidature,
};
use std::sync::Arc;
use tauri::State;

/// Charge une page de candidatures, filtrée et triée.
#[tauri::command]
pub async fn candidatures_lister_page(
    state: State<'_, AppState>,
    page: u64,
    page_size: u64,
    filtre: FiltreCandidatures,
) -> AppResult<Page<Candidature>> {
    let service = Arc::clone(&state.candidatures);
    blocking::execute(move || service.lister_page(page, page_size, &filtre)).await
}

/// Compte les candidatures par statut, pour les en-têtes de colonnes du Kanban.
#[tauri::command]
pub async fn candidatures_repartition(
    state: State<'_, AppState>,
    filtre: FiltreCandidatures,
) -> AppResult<RepartitionPipeline> {
    let service = Arc::clone(&state.candidatures);
    blocking::execute(move || service.repartition(&filtre)).await
}

/// Récupère une candidature par identifiant.
#[tauri::command]
pub async fn candidatures_obtenir(
    state: State<'_, AppState>,
    id: uuid::Uuid,
) -> AppResult<Candidature> {
    let service = Arc::clone(&state.candidatures);
    blocking::execute(move || service.obtenir(id)).await
}

/// Crée une candidature.
#[tauri::command]
pub async fn candidatures_creer(
    state: State<'_, AppState>,
    input: NouvelleCandidature,
) -> AppResult<Candidature> {
    let service = Arc::clone(&state.candidatures);
    blocking::execute(move || service.creer(&input)).await
}

/// Remplace les champs d'une candidature.
#[tauri::command]
pub async fn candidatures_modifier(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    input: NouvelleCandidature,
) -> AppResult<Candidature> {
    let service = Arc::clone(&state.candidatures);
    blocking::execute(move || service.modifier(id, &input)).await
}

/// Change le seul statut — geste du glisser-déposer du Kanban.
#[tauri::command]
pub async fn candidatures_changer_statut(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    statut: StatutCandidature,
) -> AppResult<Candidature> {
    let service = Arc::clone(&state.candidatures);
    blocking::execute(move || service.changer_statut(id, statut)).await
}

/// Supprime une candidature et, en cascade, ses relances, entretiens et historique.
#[tauri::command]
pub async fn candidatures_supprimer(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.candidatures);
    blocking::execute(move || service.supprimer(id)).await
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
pub async fn candidatures_exporter_csv(
    state: State<'_, AppState>,
    filtre: FiltreCandidatures,
    chemin: String,
) -> AppResult<u64> {
    let service = Arc::clone(&state.candidatures);
    blocking::execute(move || {
        let page = service.lister_page(1, u64::MAX, &filtre)?;
        let csv = export::vers_csv(&page.items)?;
        std::fs::write(&chemin, csv).map_err(|error| {
            tracing::error!(%error, chemin, "export CSV impossible");
            AppError::Validation("Le fichier n'a pas pu être écrit à l'emplacement choisi.".into())
        })?;
        Ok(page.total)
    })
    .await
}
