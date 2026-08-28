//! Cas de test isolé.

use super::*;

#[test]
fn test_supprimer_delegue_au_depot() {
    let svc = ContactService::new(StubRepo);
    assert!(svc.delete(uuid::Uuid::nil()).is_ok());
}
