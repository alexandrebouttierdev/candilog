//! Cas de test isolé.

use super::*;

#[test]
fn export_pdf_produit_un_document_lisible() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("lettre.pdf");
    let cover_letter = CoverLetterPdf {
        first_name: "Alex".into(),
        last_name: "Exemple".into(),
        city: Some("Rennes".into()),
        email: "alex@exemple.fr".into(),
        job_title: Some("Développeur".into()),
        company: Some("Nova".into()),
        corps: "Madame, Monsieur,\n\nJe vous adresse ma candidature.\n\nCordialement,".into(),
        ..CoverLetterPdf::default()
    };
    std::fs::write(&path, cover_letter.render_bytes().unwrap()).unwrap();
    let document = lopdf::Document::load(path).unwrap();
    assert_eq!(document.get_pages().len(), 1);
    assert_a4_media_box(&document);
}
