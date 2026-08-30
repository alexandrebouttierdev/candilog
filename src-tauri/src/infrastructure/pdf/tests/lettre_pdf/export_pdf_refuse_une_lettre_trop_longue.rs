//! Une lettre reste une page A4 ou l'export est explicitement refusé.

use super::*;

#[test]
fn export_pdf_refuse_une_lettre_trop_longue() {
    let cover_letter = CoverLetterPdf {
        name: "Alex Exemple".into(),
        city: Some("Rennes".into()),
        email: "alex@exemple.fr".into(),
        subject: "Objet : candidature".into(),
        corps: (0..200)
            .map(|_| "Un paragraphe volontairement trop long pour une page A4.".repeat(4))
            .collect::<Vec<_>>()
            .join("\n\n"),
    };

    assert!(matches!(
        cover_letter.render_bytes(),
        Err(AppError::Validation(message)) if message.to_lowercase().contains("raccourc")
    ));
}
