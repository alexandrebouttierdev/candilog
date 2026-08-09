//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_supprime_l_entreprise() {
    let repo = repo();
    let creee = repo.create(&entree("ACME")).unwrap();
    repo.delete(creee.id).unwrap();
    assert!(repo.list().unwrap().is_empty());
}
