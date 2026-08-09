//! Cas de test isolé.

use super::*;

#[test]
fn test_create_puis_list_restitue_la_relance() {
    let repo = repo();
    let cand = candidature(&repo);
    let creee = repo.create(&entree(cand, "2026-02-01")).unwrap();
    assert_eq!(creee.candidature_id, cand);
    assert_eq!(creee.type_relance, "Email");
    assert_eq!(repo.list().unwrap().len(), 1);
}
