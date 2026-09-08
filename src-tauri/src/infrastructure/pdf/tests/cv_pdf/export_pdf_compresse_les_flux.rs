//! Cas de test isolé.

use super::*;
use crate::features::documents::application::build;
use crate::features::documents::domain::{ResumeDocument, ResumeIdentity, ResumeSkillGroup};

/// Plafond de non-régression : les cinq fontes IBM Plex embarquées pèsent 952 ko brutes, et
/// `printpdf` les sérialise sans filtre. Un CV d'une page atteignait ainsi 975 ko, refusé par
/// les portails de candidature qui bornent les pièces jointes. La compression des flux
/// (`infrastructure::pdf::compress`) doit rester en place.
const MAX_PDF_BYTES: usize = 600 * 1024;

fn test_resume() -> Vec<u8> {
    build(
        &ResumeDocument {
            identity: ResumeIdentity {
                full_name: "Alex Exemple".into(),
                title: "Administrateur systèmes".into(),
                email: "alex@exemple.fr".into(),
                ..ResumeIdentity::default()
            },
            profile: "Un profil de test.".into(),
            skill_groups: vec![ResumeSkillGroup {
                id: "skills".into(),
                name: "Techniques".into(),
                items: vec!["Linux".into()],
            }],
            ..ResumeDocument::default()
        },
        None,
    )
    .render_bytes()
    .unwrap()
}

#[test]
fn export_pdf_reste_sous_le_plafond_de_poids() {
    let bytes = test_resume();

    assert!(
        bytes.len() < MAX_PDF_BYTES,
        "CV d'une page à {} octets, au-delà du plafond de {MAX_PDF_BYTES}",
        bytes.len()
    );
}

/// Le poids ne doit pas venir d'un contenu perdu : le document reste une page A4 lisible,
/// dont les flux portent un filtre de compression.
#[test]
fn export_pdf_compresse_ses_flux_sans_perdre_la_page() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cv.pdf");
    std::fs::write(&path, test_resume()).unwrap();

    let document = lopdf::Document::load(&path).unwrap();

    assert_eq!(document.get_pages().len(), 1);
    assert_a4_media_box(&document);
    let compressed = document
        .objects
        .values()
        .filter(|objet| match objet {
            lopdf::Object::Stream(flux) => flux.dict.has(b"Filter"),
            _ => false,
        })
        .count();
    assert!(
        compressed > 0,
        "aucun flux compressé : la passe de compression a disparu"
    );
}
