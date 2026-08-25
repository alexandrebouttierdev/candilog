//! Commandes Tauri des entreprises.
//!
//! Les commandes restent fines (MIGRATION.md §22) : elles reprennent le service depuis
//! l'état, délèguent, et laissent `AppError` se sérialiser en `{ code, message }`. Aucune
//! règle métier ni aucun SQL ici.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::entreprises::domain::{Entreprise, MajEntreprise, NouvelleEntreprise};
use std::sync::Arc;
use tauri::State;

/// Liste toutes les entreprises, pour alimenter un sélecteur.
#[tauri::command]
pub async fn entreprises_lister(state: State<'_, AppState>) -> AppResult<Vec<Entreprise>> {
    let service = Arc::clone(&state.entreprises);
    blocking::execute(move || service.lister()).await
}

/// Charge une page du répertoire, filtrée par recherche libre et par type.
#[tauri::command]
pub async fn entreprises_lister_page(
    state: State<'_, AppState>,
    page: u64,
    page_size: u64,
    search: String,
    company_type: Option<String>,
) -> AppResult<Page<Entreprise>> {
    let service = Arc::clone(&state.entreprises);
    blocking::execute(move || {
        service.lister_page(page, page_size, &search, company_type.as_deref())
    })
    .await
}

/// Récupère une entreprise par identifiant.
#[tauri::command]
pub async fn entreprises_obtenir(
    state: State<'_, AppState>,
    id: uuid::Uuid,
) -> AppResult<Entreprise> {
    let service = Arc::clone(&state.entreprises);
    blocking::execute(move || service.obtenir(id)).await
}

/// Liste les types d'entreprise réellement présents, pour le filtre du répertoire.
#[tauri::command]
pub async fn entreprises_lister_types(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    let service = Arc::clone(&state.entreprises);
    blocking::execute(move || service.lister_types()).await
}

/// Crée une entreprise.
#[tauri::command]
pub async fn entreprises_creer(
    state: State<'_, AppState>,
    input: NouvelleEntreprise,
) -> AppResult<Entreprise> {
    let service = Arc::clone(&state.entreprises);
    blocking::execute(move || service.creer(&input)).await
}

/// Remplace les champs d'une entreprise.
#[tauri::command]
pub async fn entreprises_modifier(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    input: MajEntreprise,
) -> AppResult<Entreprise> {
    let service = Arc::clone(&state.entreprises);
    blocking::execute(move || service.modifier(id, &input)).await
}

/// Supprime une entreprise.
#[tauri::command]
pub async fn entreprises_supprimer(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.entreprises);
    blocking::execute(move || service.supprimer(id)).await
}
