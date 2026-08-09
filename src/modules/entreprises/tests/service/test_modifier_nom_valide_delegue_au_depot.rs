//! Cas de test isolé.

use super::*;

#[test]
fn test_modifier_nom_valide_delegue_au_depot() {
    let svc = EntrepriseService::new(StubRepo);
    let e = svc.modifier(uuid::Uuid::nil(), &nouvelle("ACME")).unwrap();
    assert_eq!(e.nom, "ACME");
}
