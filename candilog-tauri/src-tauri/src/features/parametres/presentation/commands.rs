//! Frontière IPC des réglages, sauvegardes et mises à jour.

use crate::app::state::AppState;
use crate::core::errors::AppResult;
use crate::core::utils::blocking;
use crate::features::parametres::domain::{
    APropos, LlmFormulaire, MiseAJour, Parametres, ProgressionMaj,
};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// Charge les réglages, clé API comprise (coffre).
#[tauri::command]
pub async fn parametres_charger(state: State<'_, AppState>) -> AppResult<Parametres> {
    let service = Arc::clone(&state.reglages);
    blocking::execute(move || service.charger()).await
}

/// Valide et persiste les réglages. La clé API quitte SQLite pour le coffre.
#[tauri::command]
pub async fn parametres_enregistrer(
    state: State<'_, AppState>,
    parametres: Parametres,
) -> AppResult<Parametres> {
    let service = Arc::clone(&state.reglages);
    blocking::execute(move || service.enregistrer(parametres)).await
}

/// Teste le fournisseur décrit par le formulaire, sans l'enregistrer.
#[tauri::command]
pub async fn parametres_tester_connexion(
    state: State<'_, AppState>,
    llm: LlmFormulaire,
) -> AppResult<()> {
    state.reglages.tester_connexion(llm).await
}

/// Liste les modèles du fournisseur décrit par le formulaire.
#[tauri::command]
pub async fn parametres_lister_modeles(
    state: State<'_, AppState>,
    llm: LlmFormulaire,
) -> AppResult<Vec<String>> {
    state.reglages.lister_modeles(llm).await
}

/// Vide le cache des réponses IA.
#[tauri::command]
pub async fn parametres_vider_cache_ia(state: State<'_, AppState>) -> AppResult<()> {
    let service = Arc::clone(&state.reglages);
    blocking::execute(move || service.vider_cache_ia()).await
}

/// Exporte la base vers le chemin choisi dans le sélecteur natif.
#[tauri::command]
pub async fn parametres_exporter(state: State<'_, AppState>, chemin: String) -> AppResult<()> {
    let service = Arc::clone(&state.reglages);
    blocking::execute(move || service.exporter(PathBuf::from(chemin).as_path())).await
}

/// Restaure un backup validé, avec retour arrière en cas d'échec.
#[tauri::command]
pub async fn parametres_restaurer(state: State<'_, AppState>, chemin: String) -> AppResult<()> {
    let service = Arc::clone(&state.reglages);
    blocking::execute(move || service.restaurer(PathBuf::from(chemin).as_path())).await
}

/// Efface les données utilisateur, pas le référentiel des secteurs.
#[tauri::command]
pub async fn parametres_reinitialiser(state: State<'_, AppState>) -> AppResult<()> {
    let service = Arc::clone(&state.reglages);
    blocking::execute(move || service.reinitialiser()).await
}

/// Compare la version installée à la dernière release GitHub.
#[tauri::command]
pub async fn parametres_verifier_maj(state: State<'_, AppState>) -> AppResult<Option<MiseAJour>> {
    state.reglages.verifier_maj().await
}

/// Télécharge l'installeur (événement `maj-progression`) puis l'ouvre.
#[tauri::command]
pub async fn parametres_telecharger_maj(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
    nom: String,
) -> AppResult<String> {
    let notifier = move |progression: u8| {
        if let Err(error) = app.emit("maj-progression", ProgressionMaj { progression }) {
            tracing::warn!(%error, "progression de mise à jour non émise");
        }
    };
    let chemin = state.reglages.telecharger_maj(url, nom, notifier).await?;
    Ok(chemin.to_string_lossy().into_owned())
}

/// Version et nom affichés sur l'écran À propos.
#[tauri::command]
pub async fn parametres_a_propos(state: State<'_, AppState>) -> AppResult<APropos> {
    Ok(state.reglages.a_propos())
}
