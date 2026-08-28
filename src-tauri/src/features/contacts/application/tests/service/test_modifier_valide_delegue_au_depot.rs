//! Cas de test isolé.

use super::*;

#[test]
fn test_modifier_valide_delegue_au_depot() {
    let svc = ContactService::new(StubRepo);
    let c = svc
        .update(uuid::Uuid::nil(), &new("Ada", "Lovelace"))
        .unwrap();
    assert_eq!(c.first_name, "Ada");
}
