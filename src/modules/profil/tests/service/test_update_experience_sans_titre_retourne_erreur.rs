//! Cas de test isolé.

use super::*;

#[test]
fn test_update_experience_sans_titre_retourne_erreur() {
    let profil = Profile {
        experiences: vec![Experience {
            title: String::new(),
            company: "ACME".into(),
            ..Experience::default()
        }],
        ..Profile::default()
    };
    let r = service().update(&profil);
    assert!(matches!(r, Err(AppError::Validation(_))));
}
