//! Exécution des appels métier hors du fil d'événements de Tauri.

use crate::core::errors::{AppError, AppResult};

/// Exécute un appel métier bloquant sur le pool de threads dédié.
///
/// Les dépôts s'appuient sur `rusqlite`, dont l'API est synchrone : appelée directement dans
/// une commande `async`, une requête bloquerait le fil d'événements de Tauri et figerait
/// l'interface le temps de l'accès disque (MIGRATION.md §28). Toutes les commandes passent
/// donc par cette fonction.
///
/// # Errors
/// Retourne l'erreur du travail, ou `AppError::Database` si le fil s'est interrompu — ce qui
/// n'arrive qu'en cas de panique dans le métier, où `deny(clippy::unwrap_used)` rend la
/// situation improbable mais pas impossible.
pub async fn execute<T, F>(work: F) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| {
            tracing::error!(%error, "tâche métier interrompue");
            AppError::Database("Le traitement s'est interrompu de façon inattendue.".into())
        })?
}
