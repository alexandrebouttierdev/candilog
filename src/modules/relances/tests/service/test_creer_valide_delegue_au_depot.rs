//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_valide_delegue_au_depot() {
    let svc = RelanceService::new(StubRepo);
    let c = svc.creer(&nouvelle(1, "2026-07-14T10:00:00Z")).unwrap();
    assert_eq!(c.type_relance, "Email");
}
