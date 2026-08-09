//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_supprime_l_entretien() {
    let repo = repo();
    let cand = candidature(&repo);
    let cree = repo.create(&entree(cand, "2026-03-01T10:00:00Z")).unwrap();
    repo.delete(cree.id).unwrap();
    assert!(repo.list().unwrap().is_empty());
}
