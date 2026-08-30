//! Sélection et écriture sécurisées des fichiers choisis par l'utilisateur.

use crate::core::errors::{AppError, AppResult};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

fn extension_matches(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|actual| {
            extensions
                .iter()
                .any(|expected| actual.eq_ignore_ascii_case(expected.trim_start_matches('.')))
        })
}

fn validation(message: impl Into<String>) -> AppError {
    AppError::Validation(message.into())
}

fn selected_path(selection: tauri_plugin_dialog::FilePath) -> AppResult<PathBuf> {
    selection
        .into_path()
        .map_err(|_| validation("Cet emplacement n'est pas pris en charge sur votre système."))
}

/// Ouvre un dialogue natif de sauvegarde puis valide la destination sélectionnée.
///
/// # Errors
/// Retourne une erreur si l'emplacement sélectionné est invalide. Une annulation retourne
/// `Ok(None)`.
pub fn select_save_target(
    app: &AppHandle,
    title: &str,
    default_name: &str,
    filter_name: &str,
    extension: &str,
) -> AppResult<Option<PathBuf>> {
    let selected = app
        .dialog()
        .file()
        .set_title(title)
        .set_file_name(default_name)
        .add_filter(filter_name, &[extension])
        .blocking_save_file();
    selected
        .map(selected_path)
        .transpose()?
        .map(|path| validate_selected_target(&path, extension))
        .transpose()
}

/// Ouvre un dialogue natif de lecture puis valide la source sélectionnée.
///
/// # Errors
/// Retourne une erreur si le fichier sélectionné est invalide. Une annulation retourne
/// `Ok(None)`.
pub fn select_source(
    app: &AppHandle,
    title: &str,
    filter_name: &str,
    extensions: &[&str],
) -> AppResult<Option<PathBuf>> {
    let selected = app
        .dialog()
        .file()
        .set_title(title)
        .add_filter(filter_name, extensions)
        .blocking_pick_file();
    selected
        .map(selected_path)
        .transpose()?
        .map(|path| validate_selected_source(&path, extensions))
        .transpose()
}

/// Valide une destination issue d'un dialogue natif et normalise uniquement son parent.
///
/// # Errors
/// Refuse les chemins relatifs, extensions inattendues, parents inexistants, répertoires et
/// liens symboliques.
pub fn validate_selected_target(path: &Path, extension: &str) -> AppResult<PathBuf> {
    if !path.is_absolute() {
        return Err(validation("L'emplacement sélectionné n'est pas absolu."));
    }
    if !extension_matches(path, &[extension]) {
        return Err(validation(format!(
            "Le fichier sélectionné doit utiliser l'extension .{}.",
            extension.trim_start_matches('.')
        )));
    }
    let file_name = path
        .file_name()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| validation("Le nom du fichier sélectionné est invalide."))?;
    let parent = path
        .parent()
        .ok_or_else(|| validation("Le dossier de destination est invalide."))?
        .canonicalize()
        .map_err(|error| {
            AppError::Database(format!("Dossier de destination inaccessible : {error}"))
        })?;
    if !parent.is_dir() {
        return Err(validation("Le dossier de destination n'existe pas."));
    }
    let normalized = parent.join(file_name);
    match std::fs::symlink_metadata(&normalized) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(validation("Un lien symbolique ne peut pas être remplacé."))
        }
        Ok(metadata) if !metadata.is_file() => {
            Err(validation("La destination n'est pas un fichier régulier."))
        }
        Ok(_) => Ok(normalized),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(normalized),
        Err(error) => Err(AppError::Database(format!(
            "Destination inaccessible : {error}"
        ))),
    }
}

/// Valide une source issue d'un dialogue natif.
///
/// # Errors
/// Refuse les chemins relatifs, extensions inattendues et tout objet qui n'est pas un fichier
/// régulier direct.
pub fn validate_selected_source(path: &Path, extensions: &[&str]) -> AppResult<PathBuf> {
    if !path.is_absolute() {
        return Err(validation("Le fichier sélectionné n'est pas absolu."));
    }
    if !extension_matches(path, extensions) {
        return Err(validation(
            "Le format du fichier sélectionné n'est pas accepté.",
        ));
    }
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| AppError::Database(format!("Fichier source inaccessible : {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(validation(
            "La source sélectionnée doit être un fichier régulier.",
        ));
    }
    path.canonicalize()
        .map_err(|error| AppError::Database(format!("Fichier source inaccessible : {error}")))
}

struct TemporaryFileGuard {
    path: PathBuf,
    published: bool,
}

impl Drop for TemporaryFileGuard {
    fn drop(&mut self) {
        if !self.published {
            match std::fs::remove_file(&self.path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => tracing::warn!(%error, "fichier temporaire non supprimé"),
            }
        }
    }
}

/// Construit un fichier temporaire dans le dossier cible puis le publie par renommage.
///
/// # Errors
/// Retourne l'erreur de validation, de génération, de synchronisation ou de renommage. La cible
/// existante est préservée et le temporaire supprimé sur toute erreur antérieure à la publication.
pub fn atomic_write(
    target: &Path,
    extension: &str,
    build: impl FnOnce(&Path) -> AppResult<()>,
) -> AppResult<()> {
    let target = validate_selected_target(target, extension)?;
    let parent = target
        .parent()
        .ok_or_else(|| validation("Le dossier de destination est invalide."))?;
    let temporary = parent.join(format!(
        ".candilog-{}.tmp.{}",
        uuid::Uuid::new_v4(),
        extension.trim_start_matches('.')
    ));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| {
            AppError::Database(format!(
                "Création du fichier temporaire impossible : {error}"
            ))
        })?;
    drop(file);
    let mut guard = TemporaryFileGuard {
        path: temporary.clone(),
        published: false,
    };

    build(&temporary)?;
    File::options()
        .read(true)
        .write(true)
        .open(&temporary)
        .and_then(|file| file.sync_all())
        .map_err(|error| {
            AppError::Database(format!("Synchronisation du fichier impossible : {error}"))
        })?;
    std::fs::rename(&temporary, &target).map_err(|error| {
        AppError::Database(format!("Publication du fichier impossible : {error}"))
    })?;
    guard.published = true;
    if let Err(error) = File::open(parent).and_then(|directory| directory.sync_all()) {
        tracing::warn!(%error, "dossier de destination non synchronisé");
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/files/mod.rs"]
mod tests;
