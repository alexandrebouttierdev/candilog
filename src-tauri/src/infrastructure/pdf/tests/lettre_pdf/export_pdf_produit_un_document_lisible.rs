//! Cas de test isolé.

use super::*;

#[test]
fn export_pdf_produit_un_document_lisible() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("lettre.pdf");
    let cover_letter = CoverLetterPdf {
        name: "Alex Exemple".into(),
        city: Some("Rennes".into()),
        email: "alex@exemple.fr".into(),
        subject: "Objet : candidature au poste de Développeur".into(),
        corps: "Madame, Monsieur,\n\nJe vous adresse ma candidature.\n\nCordialement,".into(),
    };
    cover_letter.render_pdf(&path).unwrap();
    let document = lopdf::Document::load(path).unwrap();
    assert!(!document.get_pages().is_empty());
}
