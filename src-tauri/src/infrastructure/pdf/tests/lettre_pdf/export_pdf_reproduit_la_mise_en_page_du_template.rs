//! Cas de test isolé.

use super::*;

#[test]
fn export_pdf_reproduit_la_mise_en_page_du_template() {
    let cover_letter = CoverLetterPdf {
        first_name: "Camille".into(),
        last_name: "Rouault".into(),
        title: Some("Développeuse full-stack".into()),
        address: Some("14 rue Saint-Melaine".into()),
        city: Some("35000 Rennes".into()),
        phone: Some("06 12 34 56 78".into()),
        email: "camille.rouault@example.com".into(),
        company: Some("Groupe Ferval".into()),
        recipient: Some("Service recrutement".into()),
        recipient_address: Some("12 rue de la Monnaie, 35000 Rennes".into()),
        job_title: Some("développeuse full-stack senior".into()),
        job_reference: Some("FS-2026-114".into()),
        corps: "Madame, Monsieur,\n\nVotre offre mentionne la reprise d'un socle.".into(),
    };

    let octets = cover_letter.render_bytes().unwrap();
    let document = lopdf::Document::load_mem(&octets).unwrap();
    let texte = document.extract_text(&[1]).unwrap();

    assert!(texte.contains("Camille"));
    assert!(texte.contains("Rouault"));
    assert!(texte.contains("Groupe Ferval"));
    assert!(texte.contains("Service recrutement"));
    assert!(texte.contains("Candidature au poste de développeuse full-stack senior"));
    assert!(texte.contains("FS-2026-114"));
    assert!(texte.contains("14 rue Saint-Melaine"));
    assert!(texte.contains("curriculum"));
    assert!(!texte.contains("Objet :"));
}
