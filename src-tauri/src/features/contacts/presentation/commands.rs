//! Commandes Tauri des contacts du réseau.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use crate::core::utils::blocking;
use crate::features::contacts::domain::{Contact, MajContact, NouveauContact};
use std::sync::Arc;
use tauri::State;

/// Liste tous les contacts.
#[tauri::command]
pub async fn contacts_lister(state: State<'_, AppState>) -> AppResult<Vec<Contact>> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.lister()).await
}

/// Charge une page du réseau, filtrée par recherche libre.
#[tauri::command]
pub async fn contacts_lister_page(
    state: State<'_, AppState>,
    page: u64,
    page_size: u64,
    search: String,
) -> AppResult<Page<Contact>> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.lister_page(page, page_size, &search)).await
}

/// Récupère un contact par identifiant.
#[tauri::command]
pub async fn contacts_obtenir(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<Contact> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.obtenir(id)).await
}

/// Crée un contact.
#[tauri::command]
pub async fn contacts_creer(
    state: State<'_, AppState>,
    input: NouveauContact,
) -> AppResult<Contact> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.creer(&input)).await
}

/// Remplace les champs d'un contact.
#[tauri::command]
pub async fn contacts_modifier(
    state: State<'_, AppState>,
    id: uuid::Uuid,
    input: MajContact,
) -> AppResult<Contact> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.modifier(id, &input)).await
}

/// Supprime un contact.
#[tauri::command]
pub async fn contacts_supprimer(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.contacts);
    blocking::execute(move || service.supprimer(id)).await
}
