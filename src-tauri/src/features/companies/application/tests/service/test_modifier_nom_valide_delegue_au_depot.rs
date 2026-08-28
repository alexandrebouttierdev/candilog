//! Cas de test isolé.

use super::*;

#[test]
fn test_modifier_nom_valide_delegue_au_depot() {
    let svc = CompanyService::new(StubRepo);
    let e = svc.update(uuid::Uuid::nil(), &new("ACME")).unwrap();
    assert_eq!(e.name, "ACME");
}
