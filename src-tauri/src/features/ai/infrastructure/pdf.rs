//! Lecture locale et bornée des CV PDF.

use crate::core::errors::{AppError, AppResult};
use crate::core::files::validate_selected_source;
use std::path::{Path, PathBuf};

const MAX_PDF_BYTES: u64 = 10 * 1024 * 1024;

pub async fn extract_pdf(path: PathBuf) -> AppResult<String> {
    tauri::async_runtime::spawn_blocking(move || extract(&path))
        .await
        .map_err(|e| AppError::Validation(format!("Lecture du PDF interrompue : {e}")))?
}

fn extract(path: &Path) -> AppResult<String> {
    let path = validate_selected_source(path, &["pdf"])?;
    let metadata = std::fs::metadata(&path)
        .map_err(|_| AppError::Validation("Le fichier PDF est introuvable".into()))?;
    if metadata.len() > MAX_PDF_BYTES {
        return Err(AppError::Validation(
            "Le PDF dépasse la limite de 10 Mo".into(),
        ));
    }
    let bytes = std::fs::read(&path)
        .map_err(|_| AppError::Validation("Le fichier PDF ne peut pas être lu".into()))?;
    if !bytes.starts_with(b"%PDF-") {
        return Err(AppError::Validation(
            "Le fichier sélectionné n'est pas un PDF valide".into(),
        ));
    }
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| AppError::Validation(format!("PDF illisible : {e}")))?;
    if text.trim().is_empty() {
        return Err(AppError::Validation(
            "PDF sans texte exploitable — les documents scannés ne sont pas supportés".into(),
        ));
    }
    Ok(nettoyer(&text))
}

fn nettoyer(raw: &str) -> String {
    let mut rows = Vec::new();
    let mut vide = false;
    for line in raw.replace('\r', "").lines() {
        let row = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if row.is_empty() {
            if !vide {
                rows.push(String::new());
            }
            vide = true;
        } else {
            rows.push(row);
            vide = false;
        }
    }
    rows.join("\n").trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuse_une_extension_non_pdf() {
        assert!(extract(Path::new("cv.txt")).is_err());
    }
    #[test]
    fn nettoyage_compacte_les_espaces() {
        assert_eq!(nettoyer("Rust   Tauri\n\n\nReact"), "Rust Tauri\n\nReact");
    }
}
