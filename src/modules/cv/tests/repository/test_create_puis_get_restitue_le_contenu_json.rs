//! Cas de test isolé.

use super::*;

#[test]
fn test_create_puis_get_restitue_le_contenu_json() {
    let repo = repo();
    let contenu = serde_json::json!({"personal": {"nom": "Bouttier"}, "skills": ["Rust"]});
    let creee = repo.create("CV Rust", &contenu).unwrap();
    let relue = repo.get(creee.id).unwrap();
    assert_eq!(relue.name, "CV Rust");
    assert_eq!(relue.content, contenu);
}
