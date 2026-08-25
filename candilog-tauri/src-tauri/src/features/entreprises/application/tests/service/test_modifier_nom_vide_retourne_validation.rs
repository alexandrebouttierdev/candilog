//! Cas de test isolé.

use super::*;

#[test]
fn test_modifier_nom_vide_retourne_validation() {
    let svc = EntrepriseService::new(StubRepo);
    let r = svc.modifier(uuid::Uuid::nil(), &nouvelle("   "));
    assert!(matches!(r, Err(AppError::Validation(_))));
}
