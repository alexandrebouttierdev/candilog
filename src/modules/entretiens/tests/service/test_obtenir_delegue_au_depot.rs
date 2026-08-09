//! Cas de test isolé.

use super::*;

#[test]
fn test_obtenir_delegue_au_depot() {
    let svc = EntretienService::new(StubRepo);
    let got = svc.obtenir(uuid::Uuid::nil()).unwrap();
    assert_eq!(got.candidature_id, uuid::Uuid::from_u128(1));
}
