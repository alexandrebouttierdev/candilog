//! Cas de test isolé.

use super::*;

#[test]
fn export_pdf_produit_un_document_lisible() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cv.pdf");
    let mut cv = CvPdf::default();
    cv.name = "Alex Exemple".into();
    cv.subtitle = "Administrateur systèmes".into();
    cv.profil = "Un profil de test.".into();
    cv.skills = vec!["Linux".into()];
    cv.render_pdf(&path).unwrap();
    let document = lopdf::Document::load(path).unwrap();
    assert_eq!(document.get_pages().len(), 1);
}
