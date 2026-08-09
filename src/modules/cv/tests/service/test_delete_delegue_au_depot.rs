//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_delegue_au_depot() {
    let svc = CvVersionService::new(MockRepo::default());
    svc.delete(Uuid::nil()).unwrap();
    assert_eq!(svc.repo.deleted.lock().unwrap().len(), 1);
}
