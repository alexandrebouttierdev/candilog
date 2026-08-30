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
    std::fs::write(&path, cover_letter.render_bytes().unwrap()).unwrap();
    let document = lopdf::Document::load(path).unwrap();
    assert_eq!(document.get_pages().len(), 1);
    assert_a4_media_box(&document);
}
