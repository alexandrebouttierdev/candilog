//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_prenom_vide_retourne_validation() {
    let svc = ContactService::new(StubRepo);
    let r = svc.creer(&nouveau("  ", "Lovelace"));
    assert!(matches!(r, Err(AppError::Validation(_))));
}
