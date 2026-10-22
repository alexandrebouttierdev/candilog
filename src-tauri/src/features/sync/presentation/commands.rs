//! Commandes IPC de la synchronisation. Fines : validation des entrées, délégation au
//! service. Aucun chemin ni jeton ne traverse l'IPC vers Rust : le dossier est choisi par le
//! dialogue natif et retenu côté Rust, l'authentification se fait dans le navigateur.

use crate::app::state::AppState;
use crate::core::errors::{AppError, AppResult};
use crate::features::sync::application::SyncService;
use crate::features::sync::domain::model::{
    ConflictChoice, ProviderAvailability, RemoteProbe, SetupOutcome, SetupRequest, SyncConflict,
    SyncProviderKind, SyncStatus,
};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

/// État courant de la synchronisation.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_status(state: State<'_, AppState>) -> AppResult<SyncStatus> {
    let service = Arc::clone(&state.sync);
    crate::core::utils::blocking::execute(move || service.status()).await
}

/// Fournisseurs disponibles dans cette version.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_availability() -> AppResult<ProviderAvailability> {
    Ok(SyncService::availability())
}

/// Ouvre le dialogue natif de choix de dossier et inspecte l'emplacement. `None` si
/// l'utilisateur annule.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_choose_folder(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Option<RemoteProbe>> {
    let dialog = app.clone();
    let selected = crate::core::utils::blocking::execute(move || {
        Ok(dialog
            .dialog()
            .file()
            .set_title("Choisir le dossier de synchronisation")
            .blocking_pick_folder())
    })
    .await?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let folder = selected.into_path().map_err(|_| {
        AppError::Validation("Cet emplacement n'est pas pris en charge sur votre système.".into())
    })?;
    if !folder.is_absolute() {
        return Err(AppError::Validation("Chemin de dossier invalide.".into()));
    }
    let data_dir = state
        .db_path
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_default();
    state.sync.choose_folder(folder, &data_dir).await.map(Some)
}

/// Connecte un compte Google Drive ou OneDrive dans le navigateur du système.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_connect(
    state: State<'_, AppState>,
    provider: SyncProviderKind,
) -> AppResult<RemoteProbe> {
    if !matches!(
        provider,
        SyncProviderKind::GoogleDrive | SyncProviderKind::OneDrive
    ) {
        return Err(AppError::Validation("Fournisseur inconnu.".into()));
    }
    let open =
        |url: &str| crate::core::browser::ouvrir_url(url).map_err(|error| error.user_message());
    state.sync.connect_cloud(provider, &open).await
}

/// Abandonne une connexion en attente dans le navigateur.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_cancel_connect(state: State<'_, AppState>) -> AppResult<()> {
    state.sync.cancel_connect();
    Ok(())
}

/// Met la synchronisation en service selon le scénario choisi.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_setup(
    state: State<'_, AppState>,
    request: SetupRequest,
) -> AppResult<SetupOutcome> {
    state.sync.setup(request).await
}

/// Active ou suspend la synchronisation automatique.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_set_auto(state: State<'_, AppState>, enabled: bool) -> AppResult<()> {
    let service = Arc::clone(&state.sync);
    crate::core::utils::blocking::execute(move || service.set_auto(enabled)).await
}

/// Lance une synchronisation tout de suite.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_now(state: State<'_, AppState>) -> AppResult<()> {
    state.sync.trigger();
    Ok(())
}

/// Détache cet ordinateur de son espace, sans rien effacer.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_disconnect(state: State<'_, AppState>) -> AppResult<()> {
    state.sync.disconnect().await
}

/// Remplace les données locales par celles de l'espace ; renvoie la sauvegarde prise avant.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_restore_from_space(state: State<'_, AppState>) -> AppResult<String> {
    state.sync.restore_from_space().await
}

/// Conflits en attente.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_conflicts(state: State<'_, AppState>) -> AppResult<Vec<SyncConflict>> {
    let service = Arc::clone(&state.sync);
    crate::core::utils::blocking::execute(move || service.conflicts()).await
}

/// Tranche un conflit.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_resolve_conflict(
    state: State<'_, AppState>,
    conflict_id: String,
    choice: ConflictChoice,
) -> AppResult<()> {
    uuid::Uuid::parse_str(&conflict_id)
        .map_err(|_| AppError::Validation("Conflit inconnu.".into()))?;
    let service = Arc::clone(&state.sync);
    crate::core::utils::blocking::execute(move || service.resolve(&conflict_id, choice)).await
}

/// Tranche tous les conflits de la même façon.
#[tauri::command(rename_all = "snake_case")]
pub async fn sync_resolve_all(
    state: State<'_, AppState>,
    choice: ConflictChoice,
) -> AppResult<u32> {
    let service = Arc::clone(&state.sync);
    crate::core::utils::blocking::execute(move || service.resolve_all(choice)).await
}
