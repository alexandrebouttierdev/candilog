//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_candidature_nulle_retourne_validation() {
    let svc = RelanceService::new(StubRepo);
    let r = svc.creer(&nouvelle(0, "2026-07-14T10:00:00Z"));
    assert!(matches!(r, Err(AppError::Validation(_))));
}
