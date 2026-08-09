//! Cas de test isolé.

use super::*;

#[test]
fn test_create_puis_list_restitue_les_champs() {
    let repo = repo();
    let creee = repo.create(&entree("ACME")).unwrap();
    assert_eq!(creee.nom, "ACME");
    assert_eq!(creee.ville.as_deref(), Some("Lyon"));
    assert!(!creee.created_at.is_empty());

    let toutes = repo.list().unwrap();
    assert_eq!(toutes.len(), 1);
    assert_eq!(toutes[0].id, creee.id);
}
