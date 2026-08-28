//! Cas de test isolé.

use super::*;

#[test]
fn test_modifier_valide_delegue_au_depot() {
    let svc = ContactService::new(StubRepo);
    let c = svc
        .modifier(uuid::Uuid::nil(), &nouveau("Ada", "Lovelace"))
        .unwrap();
    assert_eq!(c.prenom, "Ada");
}
