//! Commandes Tauri des entretiens.

use crate::app::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::entretiens::domain::{Entretien, NouvelEntretien};
use std::sync::Arc;
use tauri::State;

/// Liste les entretiens d'une plage de dates, bornes incluses.
#[tauri::command]
pub async fn entretiens_lister_entre(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> AppResult<Vec<Entretien>> {
    let service = Arc::clone(&state.entretiens);
    blocking::execute(move || service.lister_entre(&from, &to)).await
}

/// Récupère un entretien par identifiant.
#[tauri::command]
pub async fn entretiens_obtenir(
    state: State<'_, AppState>,
    id: uuid::Uuid,
) -> AppResult<Entretien> {
    let service = Arc::clone(&state.entretiens);
    blocking::execute(move || service.obtenir(id)).await
}

/// Enregistre un entretien et fait passer sa candidature au statut « Entretien ».
///
/// `id` absent crée, `id` présent modifie : le chemin est unique côté dépôt, où l'écriture
/// et la mise à jour du statut sont dans la même transaction.
#[tauri::command]
pub async fn entretiens_enregistrer(
    state: State<'_, AppState>,
    id: Option<uuid::Uuid>,
    input: NouvelEntretien,
) -> AppResult<Entretien> {
    let service = Arc::clone(&state.entretiens);
    blocking::execute(move || service.enregistrer(id, &input)).await
}

/// Supprime un entretien.
#[tauri::command]
pub async fn entretiens_supprimer(state: State<'_, AppState>, id: uuid::Uuid) -> AppResult<()> {
    let service = Arc::clone(&state.entretiens);
    blocking::execute(move || service.supprimer(id)).await
}
