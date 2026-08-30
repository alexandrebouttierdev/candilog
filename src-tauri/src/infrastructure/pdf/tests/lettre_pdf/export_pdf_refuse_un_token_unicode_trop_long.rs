//! Le wrapper ne coupe jamais au milieu d'un graphème Unicode.

use super::*;

#[test]
fn export_pdf_refuse_un_token_unicode_trop_long() {
    let cover_letter = CoverLetterPdf {
        name: "Alex Exemple".into(),
        subject: "Objet : candidature".into(),
        corps: format!(
            "{}{}",
            "https://exemple.fr/chemin/".repeat(500),
            "e\u{301}👩‍💻".repeat(50)
        ),
        ..CoverLetterPdf::default()
    };

    assert!(matches!(
        cover_letter.render_bytes(),
        Err(AppError::Validation(_))
    ));
}
