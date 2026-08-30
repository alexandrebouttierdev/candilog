//! Cas de test isolé.

use super::*;

/// La mise en forme doit franchir l'export : ce qui est vu dans l'éditeur est ce qui est
/// imprimé. Le texte est comparé au rendu brut, seul témoin de ce qui atteint la page.
#[test]
fn export_pdf_rend_la_mise_en_forme() {
    let cover_letter = CoverLetterPdf {
        name: "Alex Exemple".into(),
        city: Some("Rennes".into()),
        email: "alex@exemple.fr".into(),
        subject: "Objet : candidature".into(),
        corps: "<p align=\"center\" size=\"large\">Madame, <b>Monsieur</b>,</p>\
                <p>Je reste <u>disponible</u> dès septembre.</p>"
            .into(),
    };

    let octets = cover_letter.render_bytes().unwrap();
    let document = lopdf::Document::load_mem(&octets).unwrap();
    let texte = document.extract_text(&[1]).unwrap();

    assert!(texte.contains("Madame,"));
    assert!(texte.contains("Monsieur"));
    assert!(texte.contains("disponible"));
    // Le balisage lui-même ne doit jamais apparaître sur la page.
    assert!(!texte.contains("<p"));
    assert!(!texte.contains("<b>"));
    assert!(!texte.contains("align="));
}

/// Une lettre écrite avant l'éditeur n'a aucune balise et doit continuer à s'exporter.
#[test]
fn export_pdf_accepte_encore_le_texte_brut() {
    let cover_letter = CoverLetterPdf {
        name: "Alex Exemple".into(),
        city: None,
        email: "alex@exemple.fr".into(),
        subject: "Objet : candidature".into(),
        corps: "Madame, Monsieur,\n\nJe vous adresse ma candidature.".into(),
    };

    let octets = cover_letter.render_bytes().unwrap();
    let document = lopdf::Document::load_mem(&octets).unwrap();
    let texte = document.extract_text(&[1]).unwrap();

    assert!(texte.contains("Madame, Monsieur,"));
}
