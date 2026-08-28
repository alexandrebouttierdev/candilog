//! Cas de test isolé.

use super::*;

#[test]
fn test_modifier_nom_vide_retourne_validation() {
    let svc = CompanyService::new(StubRepo);
    let r = svc.update(uuid::Uuid::nil(), &new("   "));
    assert!(matches!(r, Err(AppError::Validation(_))));
}
