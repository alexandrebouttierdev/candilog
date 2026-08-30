//! Commands Tauri des entreprises.
//!
//! Les commandes restent fines (docs/CODE_RULES.md §5) : elles reprennent le service depuis
//! l'état, délèguent, et laissent `AppError` se sérialiser en `{ code, message }`. Aucune
//! règle métier ni aucun SQL ici.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::companies::domain::{Company, CompanyFilter, CompanyUpdate, NewCompany};
use std::sync::Arc;
use tauri::State;

/// Liste toutes les entreprises, pour alimenter un sélecteur.
#[tauri::command(rename_all = "snake_case")]
pub async fn companies_list(state: State<'_, AppState>) -> AppResult<Vec<Company>> {
    let service = Arc::clone(&state.companies);
    blocking::execute(move || service.list()).await
}

/// Renvoie une page du répertoire, filtrée par recherche libre, secteur, type et taille.
#[tauri::command(rename_all = "snake_case")]
pub async fn companies_list_page(
    state: State<'_, AppState>,
    page: u64,
    page_size: u64,
    filter: CompanyFilter,
) -> AppResult<Page<Company>> {
    let service = Arc::clone(&state.companies);
    blocking::execute(move || service.list_page(page, page_size, &filter)).await
}

/// Récupère une entreprise par identifiant.
#[tauri::command(rename_all = "snake_case")]
pub async fn companies_get(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<Company> {
    let service = Arc::clone(&state.companies);
    blocking::execute(move || service.get(id)).await
}

/// Crée une entreprise.
#[tauri::command(rename_all = "snake_case")]
pub async fn companies_create(state: State<'_, AppState>, input: NewCompany) -> AppResult<Company> {
    let service = Arc::clone(&state.companies);
    blocking::execute(move || service.create(&input)).await
}

/// Remplace les champs d'une entreprise.
#[tauri::command(rename_all = "snake_case")]
pub async fn companies_update(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    input: CompanyUpdate,
) -> AppResult<Company> {
    let service = Arc::clone(&state.companies);
    blocking::execute(move || service.update(id, &input)).await
}

/// Supprime une entreprise.
#[tauri::command(rename_all = "snake_case")]
pub async fn companies_delete(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.companies);
    blocking::execute(move || service.delete(id)).await
}
