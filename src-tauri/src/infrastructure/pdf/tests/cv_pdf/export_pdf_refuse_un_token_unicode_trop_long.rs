//! Un token sans espaces est découpé par graphème au lieu de sortir horizontalement.

use super::*;

#[test]
fn export_pdf_refuse_un_token_unicode_trop_long() {
    let token = format!(
        "{}{}",
        "https://exemple.fr/chemin/".repeat(500),
        "e\u{301}👩‍💻".repeat(50)
    );
    let resume = ResumePdf {
        name: "Alex Exemple".into(),
        subtitle: "Administrateur systèmes".into(),
        profile: token,
        ..ResumePdf::default()
    };

    assert!(matches!(
        resume.render_bytes(),
        Err(AppError::Validation(_))
    ));
}
