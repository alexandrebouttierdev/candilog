//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_valide_delegue_au_depot() {
    let svc = ContactService::new(StubRepo);
    let c = svc.creer(&nouveau("Ada", "Lovelace")).unwrap();
    assert_eq!(c.nom, "Lovelace");
}
