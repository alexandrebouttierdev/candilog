//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_date_vide_retourne_validation() {
    let svc = EntretienService::new(StubRepo);
    assert!(matches!(
        svc.creer(&nouveau(1, "  ")),
        Err(AppError::Validation(_))
    ));
}
