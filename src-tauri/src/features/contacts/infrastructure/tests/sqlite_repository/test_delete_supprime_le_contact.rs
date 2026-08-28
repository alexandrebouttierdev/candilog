//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_supprime_le_contact() {
    let repo = repo();
    let cree = repo.create(&entree("Bouttier", None)).unwrap();
    repo.delete(cree.id).unwrap();
    assert!(repo.list().unwrap().is_empty());
}
