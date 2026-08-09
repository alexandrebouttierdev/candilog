//! Cas de test isolé.

use super::*;

#[test]
fn test_supprimer_delegue_au_depot() {
    let svc = CandidatureService::new(MockRepo::default());
    svc.supprimer(Uuid::nil()).unwrap();
    assert_eq!(svc.repo.deleted.lock().unwrap().len(), 1);
}
