//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_nom_vide_retourne_validation() {
    let svc = CompanyService::new(StubRepo);
    let r = svc.create(&new("   "));
    assert!(matches!(r, Err(AppError::Validation(_))));
}
