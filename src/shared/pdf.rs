//! Extraction du texte d'un fichier PDF fourni en mémoire.

use crate::shared::error::{AppError, AppResult};

/// Extrait le texte d'un PDF fourni en mémoire.
///
/// # Errors
/// `AppError::Validation` si les bytes ne forment pas un PDF lisible, ou si le
/// PDF ne contient aucun texte exploitable (cas d'un PDF scanné / image).
pub fn extract_text(bytes: &[u8]) -> AppResult<String> {
    let text = pdf_extract::extract_text_from_mem(bytes)
        .map_err(|e| AppError::Validation(format!("PDF illisible : {e}")))?;
    if text.trim().is_empty() {
        return Err(AppError::Validation(
            "PDF sans texte exploitable — les PDF scannés/images ne sont pas supportés".into(),
        ));
    }
    Ok(text)
}

/// Nettoie le texte brut d'un CV avant analyse `LLM`.
///
/// Normalise les fins de ligne, compacte les espaces internes et réduit les lignes
/// vides consécutives à une seule. Réduit le bruit et le nombre de tokens envoyés au
/// modèle (donc la latence) **sans supprimer de contenu** — utile pour un petit modèle.
#[must_use]
pub fn clean_cv_text(raw: &str) -> String {
    let normalized = raw.replace('\r', "");
    let mut out = String::with_capacity(normalized.len());
    let mut blank_run = 0_u32;
    for line in normalized.lines() {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run == 1 {
                out.push('\n');
            }
        } else {
            blank_run = 0;
            out.push_str(&line.split_whitespace().collect::<Vec<_>>().join(" "));
            out.push('\n');
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
#[path = "tests/pdf/mod.rs"]
mod tests;
