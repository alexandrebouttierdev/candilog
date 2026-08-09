//! Cas de test isolé.

use super::*;

#[test]
fn export_pdf_produit_un_document_lisible() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cv.pdf");
    CvLayout {
        name: "Alex Exemple".into(),
        headline: "Administrateur systèmes".into(),
        lines: vec!["Expérience".into(), "Linux et réseaux".into()],
    }
    .render_pdf(&path)
    .unwrap();
    let document = Document::load(path).unwrap();
    assert_eq!(document.get_pages().len(), 1);
}
