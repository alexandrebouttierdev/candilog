//! Cas de test isolé.

use super::*;
use crate::features::documents::application::build;
use crate::features::documents::domain::{ResumeDocument, ResumeIdentity, ResumeSkillGroup};

#[test]
fn export_pdf_produit_un_document_lisible() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cv.pdf");
    let resume = build(
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
    );
    std::fs::write(&path, resume.render_bytes().unwrap()).unwrap();
    let document = lopdf::Document::load(path).unwrap();
    assert_eq!(document.get_pages().len(), 1);
    assert_a4_media_box(&document);
}
