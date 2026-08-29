//! Commands Tauri des contacts du réseau.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::contacts::domain::{Contact, ContactUpdate, NewContact};
use std::sync::Arc;
use tauri::State;

/// List tous les contacts.
#[tauri::command(rename_all = "snake_case")]
pub async fn contacts_list(state: State<'_, AppState>) -> AppResult<Vec<Contact>> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.list()).await
}

/// Payload une page du réseau, filtrée par recherche libre et par rôle.
#[tauri::command(rename_all = "snake_case")]
pub async fn contacts_list_page(
    state: State<'_, AppState>,
    page: u64,
    page_size: u64,
    search: String,
    tracking_role: Option<String>,
) -> AppResult<Page<Contact>> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.list_page(page, page_size, &search, tracking_role.as_deref()))
        .await
}

/// Récupère un contact par identifiant.
#[tauri::command(rename_all = "snake_case")]
pub async fn contacts_get(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<Contact> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.get(id)).await
}

/// Crée un contact.
#[tauri::command(rename_all = "snake_case")]
pub async fn contacts_create(state: State<'_, AppState>, input: NewContact) -> AppResult<Contact> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.create(&input)).await
}

/// Remplace les champs d'un contact.
#[tauri::command(rename_all = "snake_case")]
pub async fn contacts_update(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    input: ContactUpdate,
) -> AppResult<Contact> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.update(id, &input)).await
}

/// Supprime un contact.
#[tauri::command(rename_all = "snake_case")]
pub async fn contacts_delete(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.delete(id)).await
}
