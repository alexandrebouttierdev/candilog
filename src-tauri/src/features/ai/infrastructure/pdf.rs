//! Lecture locale et bornée des CV PDF.

use crate::core::errors::{AppError, AppResult};
use crate::core::files::validate_selected_source;
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_PDF_BYTES: u64 = 10 * 1024 * 1024;

pub async fn extract_pdf(path: PathBuf) -> AppResult<String> {
    tauri::async_runtime::spawn_blocking(move || extract(&path))
        .await
        .map_err(|error| {
            tracing::error!(%error, "lecture du PDF interrompue");
            AppError::Validation("La lecture du PDF a été interrompue.".into())
        })?
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
    let text = match extraire_ordre_mise_en_page(&path) {
        Some(text) => text,
        None => pdf_extract::extract_text_from_mem(&bytes).map_err(|error| {
            // Le détail vient de la bibliothèque d'extraction, en anglais : il sert au
            // diagnostic, pas à l'utilisateur (`docs/CODE_RULES.md` §13).
            tracing::warn!(%error, "extraction du texte du PDF impossible");
            AppError::Validation(
                "Ce PDF n'a pas pu être lu. Exportez-le à nouveau depuis votre traitement de \
                 texte, ou choisissez un autre fichier."
                    .into(),
            )
        })?,
    };
    if text.trim().is_empty() {
        return Err(AppError::Validation(
            "PDF sans texte exploitable — les documents scannés ne sont pas supportés".into(),
        ));
    }
    Ok(nettoyer(&text))
}

/// Texte dans l'ordre de mise en page, via `pdftotext -layout`.
///
/// `pdf-extract` suit l'ordre du flux de contenu : une colonne peinte en premier sort
/// avant l'autre, même si elle est à droite. Le modèle et le recadrage voient donc le
/// même texte, recopié tel quel, sans découpe ni reformulation. `None` si Poppler est
/// absent ou échoue : l'appelant retombe alors sur l'extracteur de flux.
fn extraire_ordre_mise_en_page(path: &Path) -> Option<String> {
    let sortie = Command::new("pdftotext")
        .args(["-layout", "-enc", "UTF-8", "-eol", "unix", "-nopgbrk"])
        .arg(path)
        .arg("-")
        .output()
        .ok()?;
    if !sortie.status.success() {
        tracing::warn!("pdftotext -layout a échoué, repli sur l'extracteur de flux");
        return None;
    }
    let texte = String::from_utf8(sortie.stdout).ok()?;
    if texte.trim().is_empty() {
        return None;
    }
    Some(texte)
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

    /// Deux colonnes fictives. Le flux peint la colonne droite d'abord ; la mise en
    /// page doit quand même lire la colonne gauche d'abord. Aucun nom réel.
    #[test]
    fn lit_deux_colonnes_dans_l_ordre_de_mise_en_page() {
        let dir = tempfile::tempdir().expect("dossier temporaire");
        let path = dir.path().join("colonnes.pdf");
        std::fs::write(&path, pdf_colonnes_fictives()).expect("écrire le PDF fictif");
        let flux = pdf_extract::extract_text_from_mem(&std::fs::read(&path).unwrap())
            .expect("extracteur de flux");
        let flux = nettoyer(&flux);
        let mise_en_page = extract(&path).expect("extraction");
        let zephyr_flux = flux.find("ZEPHYR").expect("ZEPHYR dans le flux");
        let alpha_flux = flux.find("ALPHA").expect("ALPHA dans le flux");
        assert!(
            zephyr_flux < alpha_flux,
            "le flux doit peindre la colonne droite en premier"
        );
        let alpha = mise_en_page.find("ALPHA").expect("ALPHA");
        let zephyr = mise_en_page.find("ZEPHYR").expect("ZEPHYR");
        let beta = mise_en_page.find("BETA").expect("BETA");
        let atelier = mise_en_page.find("ATELIER").expect("ATELIER");
        assert!(alpha < zephyr, "colonne gauche avant la colonne droite");
        assert!(beta < atelier, "ligne basse : gauche avant droite");
        assert!(
            mise_en_page.contains("ALPHA")
                && mise_en_page.contains("BETA")
                && mise_en_page.contains("ZEPHYR")
                && mise_en_page.contains("ATELIER")
        );
    }

    fn pdf_colonnes_fictives() -> Vec<u8> {
        // Colonne droite peinte avant la gauche, pour distinguer flux et mise en page.
        let stream = b"BT\n/F1 12 Tf\n320 700 Td (ZEPHYR) Tj\n0 -24 Td (ATELIER) Tj\nET\nBT\n/F1 12 Tf\n72 700 Td (ALPHA) Tj\n0 -24 Td (BETA) Tj\nET\n";
        let objects: [&[u8]; 5] = [
            b"<< /Type /Catalog /Pages 2 0 R >>",
            b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>",
            &[],
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        ];
        let contenu = format!("<< /Length {} >>\nstream\n", stream.len());
        let mut out = Vec::from(b"%PDF-1.4\n");
        let mut offsets = vec![0];
        for (index, body) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
            if index == 3 {
                out.extend_from_slice(contenu.as_bytes());
                out.extend_from_slice(stream);
                out.extend_from_slice(b"\nendstream\n");
            } else {
                out.extend_from_slice(body);
                out.extend_from_slice(b"\n");
            }
            out.extend_from_slice(b"endobj\n");
        }
        let xref = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
        out.extend_from_slice(b"0000000000 65535 f \n");
        for offset in &offsets[1..] {
            out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        out
    }
}
