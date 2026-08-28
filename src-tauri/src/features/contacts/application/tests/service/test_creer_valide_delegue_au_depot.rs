//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_valide_delegue_au_depot() {
    let svc = ContactService::new(StubRepo);
    let c = svc.create(&new("Ada", "Lovelace")).unwrap();
    assert_eq!(c.name, "Lovelace");
}
