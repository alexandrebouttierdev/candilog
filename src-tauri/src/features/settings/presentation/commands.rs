//! Frontière IPC des réglages, sauvegardes et mises à jour.

use crate::app::state::AppState;
use crate::core::errors::AppResult;
use crate::core::files::{atomic_write, select_save_target, select_source};
use crate::core::utils::blocking;
use crate::features::settings::domain::{
    About, LlmForm, ResetOutcome, Settings, UpdateInfo, UpdateProgress,
};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// Payload les réglages, clé API comprise (coffre).
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_load(state: State<'_, AppState>) -> AppResult<Settings> {
    let service = Arc::clone(&state.settings);
    blocking::execute(move || service.load()).await
}

/// Valide et persiste les réglages. La clé API quitte SQLite pour le coffre.
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_save(
    state: State<'_, AppState>,
    settings: Settings,
    api_key: Option<String>,
) -> AppResult<Settings> {
    let service = Arc::clone(&state.settings);
    blocking::execute(move || service.save(settings, api_key)).await
}

/// Supprime la clé API du coffre natif sans modifier les autres réglages.
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_clear_api_key(state: State<'_, AppState>) -> AppResult<()> {
    let service = Arc::clone(&state.settings);
    blocking::execute(move || service.clear_api_key()).await
}

/// Teste le fournisseur décrit par le formulaire, sans l'enregistrer.
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_test_connection(
    state: State<'_, AppState>,
    llm: LlmForm,
    api_key: Option<String>,
) -> AppResult<()> {
    state.settings.test_connection(llm, api_key).await
}

/// List les modèles du fournisseur décrit par le formulaire.
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_list_models(
    state: State<'_, AppState>,
    llm: LlmForm,
    api_key: Option<String>,
) -> AppResult<Vec<String>> {
    state.settings.list_models(llm, api_key).await
}

/// Exporte la base vers le chemin choisi dans le sélecteur natif.
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_export(app: AppHandle, state: State<'_, AppState>) -> AppResult<bool> {
    let Some(cible) = select_save_target(
        &app,
        "Exporter une sauvegarde Candilog",
        "candilog.sqlite",
        "Base SQLite",
        "sqlite",
    )?
    else {
        return Ok(false);
    };
    let service = Arc::clone(&state.settings);
    blocking::execute(move || {
        atomic_write(&cible, "sqlite", |temporaire| service.export(temporaire))?;
        Ok(true)
    })
    .await
}

/// Restaure un backup validé, avec retour arrière en cas d'échec.
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_restore(app: AppHandle, state: State<'_, AppState>) -> AppResult<bool> {
    let Some(source) = select_source(
        &app,
        "Restaurer une sauvegarde Candilog",
        "Sauvegarde Candilog",
        &["sqlite", "bak"],
    )?
    else {
        return Ok(false);
    };
    let service = Arc::clone(&state.settings);
    blocking::execute(move || {
        service.restore(&source)?;
        Ok(true)
    })
    .await
}

/// Efface les données utilisateur, pas le référentiel des secteurs.
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_reset(state: State<'_, AppState>) -> AppResult<ResetOutcome> {
    let service = Arc::clone(&state.settings);
    blocking::execute(move || service.reset()).await
}

/// Compare la version installée à la dernière release GitHub.
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_check_update(state: State<'_, AppState>) -> AppResult<Option<UpdateInfo>> {
    state.settings.check_update().await
}

/// Télécharge l'installeur (événement `update-progress`), vérifie son empreinte puis l'ouvre.
///
/// Sans paramètre : l'asset est résolu côté Rust depuis l'API GitHub. Laisser le frontend
/// désigner l'URL et le nom du fichier revenait à lui laisser choisir ce que le lanceur
/// système exécuterait (`docs/CODE_RULES.md` §14).
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_download_update(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<String> {
    let notifier = move |progress: u8| {
        if let Err(error) = app.emit("update-progress", UpdateProgress { progress }) {
            tracing::warn!(%error, "progression de mise à jour non émise");
        }
    };
    let path = state.settings.download_update(notifier).await?;
    Ok(path.to_string_lossy().into_owned())
}

/// Version et nom affichés sur l'écran À propos.
#[tauri::command(rename_all = "snake_case")]
pub async fn settings_about(state: State<'_, AppState>) -> AppResult<About> {
    Ok(state.settings.about())
}

/// Ouvre un lien externe dans le navigateur du système.
///
/// Seule commande non préfixée par sa feature : elle sert les offres d'emploi, les sites
/// d'entreprise et les profils LinkedIn autant que les réglages. Elle vit ici parce que ce
/// module porte déjà les concerns système de l'application (export, restauration, mises à
/// jour), et la validation reste dans `core::browser`.
#[tauri::command(rename_all = "snake_case")]
pub async fn open_external_url(url: String) -> AppResult<()> {
    blocking::execute(move || crate::core::browser::ouvrir_url(&url)).await
}
