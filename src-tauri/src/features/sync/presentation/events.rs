//! Événement d'état de la synchronisation, émis à chaque changement.

use crate::features::sync::application::SyncService;
use crate::features::sync::domain::model::SyncStatus;
use tauri::{AppHandle, Emitter};

/// Nom de l'événement écouté par l'interface.
pub const SYNC_STATUS_EVENT: &str = "sync://status";

/// Relie le service à l'interface : chaque changement d'état est poussé à la fenêtre.
pub fn install(app: &AppHandle, service: &SyncService) {
    let handle = app.clone();
    service.set_listener(Box::new(move |status: &SyncStatus| {
        if let Err(error) = handle.emit(SYNC_STATUS_EVENT, status) {
            tracing::warn!(%error, "état de synchronisation non émis");
        }
    }));
}
