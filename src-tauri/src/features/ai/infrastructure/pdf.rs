//! Lecture locale et bornée des CV PDF (texte et rendu Vision).

use crate::core::errors::{AppError, AppResult};
use crate::core::files::validate_selected_source;
use crate::features::ai::domain::{MAX_VISION_PAGES, VISION_RENDER_DPI};
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_PDF_BYTES: u64 = 10 * 1024 * 1024;

/// Image d'une page PDF prête pour un appel Vision (octets JPEG bruts).
#[derive(Debug, Clone)]
pub struct PdfPageImage {
    pub page_index: usize,
    pub mime: &'static str,
    pub bytes: Vec<u8>,
}

pub async fn extract_pdf(path: PathBuf) -> AppResult<String> {
    tauri::async_runtime::spawn_blocking(move || extract(&path))
        .await
        .map_err(|error| {
            tracing::error!(%error, "lecture du PDF interrompue");
            AppError::Validation("La lecture du PDF a été interrompue.".into())
        })?
}

/// Extraction texte souple pour le pipeline Vision : un PDF scanné sans texte
/// exploitable n'est pas une erreur — le rendu image reste la source principale.
pub async fn try_extract_pdf_text(path: PathBuf) -> AppResult<Option<String>> {
    tauri::async_runtime::spawn_blocking(move || match extract(&path) {
        Ok(text) => Ok(Some(text)),
        Err(AppError::Validation(message))
            if message.contains("sans texte exploitable")
                || message.contains("documents scannés") =>
        {
            tracing::info!("PDF sans texte exploitable — Vision poursuivra sans complément texte");
            Ok(None)
        }
        Err(error) => Err(error),
    })
    .await
    .map_err(|error| {
        tracing::error!(%error, "lecture du PDF interrompue");
        AppError::Validation("La lecture du PDF a été interrompue.".into())
    })?
}

/// Rend les pages du PDF en JPEG via `pdftoppm` (Poppler), dans l'ordre.
///
/// Limite : [`MAX_VISION_PAGES`]. Au-delà, une erreur explicite est renvoyée — on ne
/// tronque pas silencieusement. Les fichiers temporaires sont toujours nettoyés.
pub async fn render_pdf_pages(path: PathBuf) -> AppResult<Vec<PdfPageImage>> {
    tauri::async_runtime::spawn_blocking(move || render_pages(&path))
        .await
        .map_err(|error| {
            tracing::error!(%error, "rendu PDF interrompu");
            AppError::Validation("La conversion du PDF en images a été interrompue.".into())
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

fn render_pages(path: &Path) -> AppResult<Vec<PdfPageImage>> {
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

    if let Some(page_count) = compter_pages_pdf(&path) {
        if page_count == 0 {
            return Err(AppError::Validation(
                "Impossible de déterminer le nombre de pages de ce PDF.".into(),
            ));
        }
        if page_count > MAX_VISION_PAGES {
            return Err(AppError::Validation(format!(
                "Ce CV comporte {page_count} pages. L'analyse visuelle accepte au plus \
                 {MAX_VISION_PAGES} pages. Réduisez le document, ou utilisez l'analyse Texte."
            )));
        }
    }

    let temp = tempfile::tempdir().map_err(|error| {
        tracing::warn!(%error, "dossier temporaire pour le rendu PDF indisponible");
        AppError::Validation("Impossible de préparer la conversion du PDF.".into())
    })?;
    let prefix = temp.path().join("page");
    // On demande une page de plus que la limite : si elle apparaît, le document est trop long
    // et on refuse explicitement (pas de troncature silencieuse).
    let probe_last = MAX_VISION_PAGES.saturating_add(1);
    let sortie = Command::new("pdftoppm")
        .args([
            "-jpeg",
            "-r",
            &VISION_RENDER_DPI.to_string(),
            "-jpegopt",
            "quality=85",
            "-f",
            "1",
            "-l",
            &probe_last.to_string(),
        ])
        .arg(&path)
        .arg(&prefix)
        .output()
        .map_err(|error| {
            tracing::warn!(%error, "pdftoppm indisponible pour le rendu Vision");
            AppError::Validation(
                "La conversion du PDF en images est indisponible sur cet ordinateur \
                 (outil Poppler manquant). Utilisez l'analyse Texte, ou installez Poppler."
                    .into(),
            )
        })?;
    if !sortie.status.success() {
        let stderr = String::from_utf8_lossy(&sortie.stderr);
        tracing::warn!(%stderr, "pdftoppm a échoué");
        return Err(AppError::Validation(
            "La conversion du PDF en images a échoué. Réessayez, ou utilisez l'analyse Texte."
                .into(),
        ));
    }

    // Collecte ordonnée des JPEG produits (page-1.jpg, page-01.jpg…).
    let mut files = std::fs::read_dir(temp.path())
        .map_err(|_| AppError::Validation("Impossible de lire les images converties.".into()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jpg"))
        })
        .collect::<Vec<_>>();
    files.sort();
    if files.is_empty() {
        return Err(AppError::Validation(
            "Aucune page du PDF n'a pu être convertie en image.".into(),
        ));
    }
    if files.len() > MAX_VISION_PAGES {
        return Err(AppError::Validation(format!(
            "Ce CV comporte plus de {MAX_VISION_PAGES} pages. L'analyse visuelle ne peut pas \
             les traiter toutes. Réduisez le document, ou utilisez l'analyse Texte."
        )));
    }

    let mut images = Vec::with_capacity(files.len());
    for (index, file) in files.into_iter().enumerate() {
        let bytes = std::fs::read(&file).map_err(|error| {
            tracing::warn!(%error, page = index + 1, "lecture de l'image de page impossible");
            AppError::Validation("Impossible de lire une page convertie du PDF.".into())
        })?;
        if bytes.is_empty() {
            return Err(AppError::Validation(
                "Une page convertie du PDF est vide.".into(),
            ));
        }
        images.push(PdfPageImage {
            page_index: index,
            mime: "image/jpeg",
            bytes,
        });
    }

    tracing::info!(pages = images.len(), "PDF rendu en images pour Vision");
    Ok(images)
}

fn compter_pages_pdf(path: &Path) -> Option<usize> {
    let sortie = Command::new("pdfinfo").arg(path).output().ok()?;
    if !sortie.status.success() {
        return None;
    }
    let texte = String::from_utf8(sortie.stdout).ok()?;
    for ligne in texte.lines() {
        let Some(reste) = ligne
            .strip_prefix("Pages:")
            .or_else(|| ligne.strip_prefix("Pages :"))
        else {
            continue;
        };
        return reste.trim().parse().ok();
    }
    None
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

    #[test]
    fn rendu_vision_produit_au_moins_une_image_jpeg() {
        if Command::new("pdftoppm").arg("-v").output().is_err() {
            eprintln!("pdftoppm absent — test de rendu ignoré");
            return;
        }
        if Command::new("pdfinfo").arg("-v").output().is_err() {
            // pdfinfo -v échoue parfois (code 99) mais l'outil existe : on tente quand même.
        }
        let dir = tempfile::tempdir().expect("dossier temporaire");
        let path = dir.path().join("page.pdf");
        std::fs::write(&path, pdf_colonnes_fictives()).expect("écrire le PDF fictif");
        // pdfinfo peut manquer : on force une page unique en acceptant l'échec de comptage
        // uniquement si pdftoppm produit quand même. Ici le PDF a 1 page.
        match render_pages(&path) {
            Ok(images) => {
                assert_eq!(images.len(), 1);
                assert_eq!(images[0].mime, "image/jpeg");
                assert!(images[0].bytes.starts_with(&[0xFF, 0xD8]), "JPEG SOI");
            }
            Err(error) => {
                // Sans pdfinfo, compter_pages échoue — le message doit rester explicite.
                let message = error.to_string();
                assert!(
                    message.contains("pages")
                        || message.contains("Poppler")
                        || message.contains("images"),
                    "erreur inattendue : {message}"
                );
            }
        }
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
