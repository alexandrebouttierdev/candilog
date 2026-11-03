//! Matérialisation du CV de référence du benchmark utilisateur.
//!
//! Le scoring du benchmark vit dans `domain::user_benchmark` ; l'écriture du PDF sur le
//! disque est une opération d'infrastructure (`docs/CODE_RULES.md` §5) et vit donc ici.

use crate::features::ai::domain::USER_BENCHMARK_VERSION;
use std::path::PathBuf;

/// PDF de référence compilé dans le binaire — `CARGO_MANIFEST_DIR` n'existe pas
/// sur la machine de l'utilisateur après installation.
const BENCHMARK_PDF_BYTES: &[u8] = include_bytes!("../../../../resources/CV_BENCHMARK.pdf");

/// Matérialise le PDF de benchmark dans un fichier temporaire stable.
///
/// Les extracteurs (`pdftotext`, `pdftoppm`, `pdf-extract`) travaillent sur des chemins.
/// Le contenu vient du binaire : après installation, `resources/` du dépôt n'est plus là
/// et n'est pas non plus dans le bundle Tauri.
///
/// # Errors
/// Message technique, destiné au journal, si le dossier ou le fichier ne peut être écrit.
pub fn benchmark_pdf_path() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("candilog-user-benchmark");
    std::fs::create_dir_all(&dir).map_err(|error| {
        format!(
            "impossible de préparer le dossier du CV de benchmark ({}): {error}",
            dir.display()
        )
    })?;
    let path = dir.join(format!("CV_BENCHMARK-v{USER_BENCHMARK_VERSION}.pdf"));
    let needs_write = match std::fs::metadata(&path) {
        Ok(meta) => meta.len() as usize != BENCHMARK_PDF_BYTES.len(),
        Err(_) => true,
    };
    if needs_write {
        let partial = path.with_extension("pdf.partial");
        std::fs::write(&partial, BENCHMARK_PDF_BYTES).map_err(|error| {
            format!(
                "impossible d'écrire le CV de benchmark ({}): {error}",
                partial.display()
            )
        })?;
        std::fs::rename(&partial, &path).map_err(|error| {
            format!(
                "impossible de publier le CV de benchmark ({}): {error}",
                path.display()
            )
        })?;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Régression : en installation, `CARGO_MANIFEST_DIR` pointe vers la machine de build,
    /// pas vers le disque de l'utilisateur. Le PDF doit donc être extrait du binaire.
    #[test]
    fn pdf_benchmark_est_extrait_vers_un_fichier_temporaire() {
        let path = benchmark_pdf_path().expect("matérialisation du PDF de benchmark");
        assert!(
            path.starts_with(std::env::temp_dir()),
            "attendu sous {:?}, obtenu {} — le chemin source compile-time n'existe pas après installation",
            std::env::temp_dir(),
            path.display()
        );
        let bytes = std::fs::read(&path).expect("lecture du PDF matérialisé");
        assert!(
            bytes.starts_with(b"%PDF-"),
            "le fichier matérialisé doit être un PDF"
        );
        assert_eq!(bytes.as_slice(), BENCHMARK_PDF_BYTES);
    }
}
