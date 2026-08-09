//! Cas de test isolé.

use super::*;

#[test]
fn test_list_ne_renvoie_que_les_resumes_les_plus_recents_d_abord() {
    let repo = repo();
    repo.create("Ancienne", &serde_json::json!({})).unwrap();
    repo.create("Récente", &serde_json::json!({})).unwrap();
    let resumes = repo.list().unwrap();
    assert_eq!(resumes.len(), 2);
    assert_eq!(resumes[0].name, "Récente");
}
