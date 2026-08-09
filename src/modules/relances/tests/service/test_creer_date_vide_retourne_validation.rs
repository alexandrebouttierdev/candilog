//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_date_vide_retourne_validation() {
    let svc = RelanceService::new(StubRepo);
    let r = svc.creer(&nouvelle(1, "  "));
    assert!(matches!(r, Err(AppError::Validation(_))));
}
