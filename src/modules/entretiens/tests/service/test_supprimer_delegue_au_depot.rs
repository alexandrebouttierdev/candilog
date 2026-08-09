//! Cas de test isolé.

use super::*;

#[test]
fn test_supprimer_delegue_au_depot() {
    let svc = EntretienService::new(StubRepo);
    assert!(svc.supprimer(uuid::Uuid::nil()).is_ok());
}
