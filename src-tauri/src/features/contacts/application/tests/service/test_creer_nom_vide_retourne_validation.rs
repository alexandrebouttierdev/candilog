//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_nom_vide_retourne_validation() {
    let svc = ContactService::new(StubRepo);
    let r = svc.create(&new("Ada", "  "));
    assert!(matches!(r, Err(AppError::Validation(_))));
}
