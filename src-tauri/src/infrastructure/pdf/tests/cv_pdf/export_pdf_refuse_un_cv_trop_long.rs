//! Un CV excessif doit être refusé, jamais tronqué ni placé hors page.

use super::*;

#[test]
fn export_pdf_refuse_un_cv_trop_long() {
    let resume = ResumePdf {
        name: "Alex Exemple".into(),
        subtitle: "Administrateur systèmes".into(),
        profile: "Profil".into(),
        experiences: (0..40)
            .map(|index| ResumeExperience {
                title: format!("Expérience {index}"),
                company: "Entreprise".into(),
                bullets: vec!["Description très détaillée ".repeat(100)],
                ..ResumeExperience::default()
            })
            .collect(),
        ..ResumePdf::default()
    };

    assert!(matches!(
        resume.render_bytes(),
        Err(AppError::Validation(message)) if message.to_lowercase().contains("raccourc")
    ));
}
