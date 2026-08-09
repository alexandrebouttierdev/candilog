//! Cas de test isolé.

use super::*;

#[test]
fn test_update_email_invalide_retourne_erreur() {
    let profil = Profile {
        personal: PersonalInfo {
            email: "pas-un-email".into(),
            ..PersonalInfo::default()
        },
        ..Profile::default()
    };
    let r = service().update(&profil);
    assert!(matches!(r, Err(AppError::Validation(_))));
}
