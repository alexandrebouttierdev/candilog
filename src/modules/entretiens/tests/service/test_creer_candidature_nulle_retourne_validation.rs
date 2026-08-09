//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_candidature_nulle_retourne_validation() {
    let svc = EntretienService::new(StubRepo);
    assert!(matches!(
        svc.creer(&nouveau(0, "2026-07-20T09:00:00Z")),
        Err(AppError::Validation(_))
    ));
}
