//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_nom_vide_retourne_validation() {
    let svc = EntrepriseService::new(StubRepo);
    let r = svc.creer(&nouvelle("   "));
    assert!(matches!(r, Err(AppError::Validation(_))));
}
