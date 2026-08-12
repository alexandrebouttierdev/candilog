use super::*;

#[test]
fn test_update_formation_vide_retourne_erreur() {
    let mut profile = Profile::default();
    profile.education.push(Default::default());
    let result = service().update(&profile);
    assert!(matches!(result, Err(AppError::Validation(_))));
}
