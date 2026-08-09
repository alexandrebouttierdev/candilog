//! Cas de test isolé.

use super::*;

#[test]
fn test_lister_delegue_au_depot() {
    let svc = EntretienService::new(StubRepo);
    assert_eq!(svc.lister().unwrap().len(), 1);
}
