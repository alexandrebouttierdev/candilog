//! Commands Tauri des entreprises.
//!
//! Les commandes restent fines (docs/CODE_RULES.md §5) : elles reprennent le service depuis
//! l'état, délèguent, et laissent `AppError` se sérialiser en `{ code, message }`. Aucune
//! règle métier ni aucun SQL ici.

use crate::app::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::core::files::{atomic_write, select_save_target, validate_selected_target};
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::companies::application::export::{companies_csv, contacts_file_name};
use crate::features::companies::domain::{Company, CompanyFilter, CompanyUpdate, NewCompany};
use crate::features::contacts::application::export::contacts_csv;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, State};

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

/// Bilan d'un export des relations : ce qui a été écrit, et où.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "companies.ts")]
pub struct RelationsExport {
    #[ts(type = "number")]
    pub companies: u64,
    #[ts(type = "number")]
    pub contacts: u64,
    pub companies_file: String,
    pub contacts_file: String,
}

/// Exporte les relations en deux CSV distincts (`DECISIONS.md` E10) : l'utilisateur choisit
/// le fichier des entreprises, celui des contacts est écrit dans le même dossier. L'export
/// n'est pas filtré : aucun filtre ne s'applique à l'écran Relations dans son ensemble.
#[tauri::command(rename_all = "snake_case")]
pub async fn relations_export_csv(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Option<RelationsExport>> {
    let today = chrono::Local::now().format("%Y-%m-%d");
    let Some(companies_target) = select_save_target(
        &app,
        "Exporter les relations",
        &format!("entreprises-{today}.csv"),
        "Fichier CSV",
        "csv",
    )?
    else {
        return Ok(None);
    };
    let companies_file = companies_target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::Validation("Le nom du fichier choisi est invalide.".into()))?
        .to_owned();
    let contacts_file = contacts_file_name(&companies_file);
    let contacts_target =
        validate_selected_target(&companies_target.with_file_name(&contacts_file), "csv")?;
    let companies_service = Arc::clone(&state.companies);
    let contacts_service = Arc::clone(&state.contacts);
    blocking::execute(move || {
        let companies = companies_service.list()?;
        let contacts = contacts_service.list()?;
        let write = |target: &std::path::Path, csv: String| {
            atomic_write(target, "csv", |temporaire| {
                std::fs::write(temporaire, &csv).map_err(|error| {
                    tracing::error!(%error, "export CSV des relations impossible");
                    AppError::Validation(
                        "Le fichier n'a pas pu être écrit à l'emplacement choisi.".into(),
                    )
                })
            })
        };
        write(&companies_target, companies_csv(&companies)?)?;
        write(&contacts_target, contacts_csv(&contacts)?)?;
        Ok(Some(RelationsExport {
            companies: companies.len() as u64,
            contacts: contacts.len() as u64,
            companies_file,
            contacts_file,
        }))
    })
    .await
}
