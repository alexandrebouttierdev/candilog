//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_date_vide_retourne_validation() {
    let svc = CandidatureService::new(MockRepo::default());
    let mut candidature = input("Dev");
    candidature.date_envoi.clear();
    assert!(matches!(
        svc.creer(&candidature),
        Err(AppError::Validation(_))
    ));
}
