//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_supprime_la_relance() {
    let repo = repo();
    let cand = candidature(&repo);
    let creee = repo.create(&entree(cand, "2026-02-01")).unwrap();
    repo.delete(creee.id).unwrap();
    assert!(repo.list().unwrap().is_empty());
}
