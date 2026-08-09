//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_poste_vide_retourne_validation() {
    let svc = CandidatureService::new(MockRepo::default());
    let r = svc.creer(&input("   "));
    assert!(matches!(r, Err(AppError::Validation(_))));
}
