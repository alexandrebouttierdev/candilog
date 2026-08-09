//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_supprime_la_version() {
    let repo = repo();
    let creee = repo.create("CV", &serde_json::json!({})).unwrap();
    repo.delete(creee.id).unwrap();
    assert!(repo.list().unwrap().is_empty());
}
