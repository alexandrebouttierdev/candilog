//! Cas de test isolé.

use super::*;

#[test]
fn export_pdf_produit_un_document_lisible() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("lettre.pdf");
    let lettre = LettrePdf {
        nom: "Alex Exemple".into(),
        ville: Some("Rennes".into()),
        email: "alex@exemple.fr".into(),
        objet: "Objet : candidature au poste de Développeur".into(),
        corps: "Madame, Monsieur,\n\nJe vous adresse ma candidature.\n\nCordialement,".into(),
    };
    lettre.render_pdf(&path).unwrap();
    let document = lopdf::Document::load(path).unwrap();
    assert!(!document.get_pages().is_empty());
}
