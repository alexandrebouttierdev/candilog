//! Lecture du presse-papiers système.
//!
//! La webview n'expose pas `navigator.clipboard.readText` : comme les dialogues de fichier,
//! cet accès au système passe par le natif, seul endroit où il est réellement disponible.

use crate::core::errors::{AppError, AppResult};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Nombre de caractères au-delà duquel un collage est refusé.
///
/// Aligné sur `MAX_SOURCE_CHARS` : au-delà, le texte serait de toute façon rejeté par
/// l'analyse, autant le dire tout de suite plutôt que de figer un champ de saisie.
const MAX_CLIPBOARD_CHARS: usize = 50_000;

/// Texte du presse-papiers, borné.
///
/// # Errors
/// Retourne `AppError::Validation` si le presse-papiers ne contient pas de texte lisible ou
/// si le contenu dépasse `MAX_CLIPBOARD_CHARS`.
pub fn read_text(app: &AppHandle) -> AppResult<String> {
    let text = app.clipboard().read_text().map_err(|error| {
        tracing::debug!(%error, "presse-papiers illisible");
        AppError::Validation("Le presse-papiers ne contient pas de texte.".into())
    })?;
    if text.chars().count() > MAX_CLIPBOARD_CHARS {
        return Err(AppError::Validation(format!(
            "Le presse-papiers dépasse {MAX_CLIPBOARD_CHARS} caractères."
        )));
    }
    Ok(text)
}
