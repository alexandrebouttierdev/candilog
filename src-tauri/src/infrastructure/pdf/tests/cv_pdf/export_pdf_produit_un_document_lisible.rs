//! Cas de test isolé.

use super::*;

#[test]
fn export_pdf_produit_un_document_lisible() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cv.pdf");
    let resume = ResumePdf {
        name: "Alex Exemple".into(),
        subtitle: "Administrateur systèmes".into(),
        profile: "Un profil de test.".into(),
        skills: vec!["Linux".into()],
        ..ResumePdf::default()
    };
    resume.render_pdf(&path).unwrap();
    let document = lopdf::Document::load(path).unwrap();
    assert_eq!(document.get_pages().len(), 1);
}
