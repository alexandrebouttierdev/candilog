//! Cas de test isolé.

use super::*;

#[test]
fn test_create_puis_get_restitue_le_type_et_le_lieu() {
    let repo = repo();
    let cand = candidature(&repo);
    let cree = repo.create(&entree(cand, "2026-03-01T10:00:00Z")).unwrap();
    let relu = repo.get(cree.id).unwrap();
    assert_eq!(relu.type_entretien, TypeEntretien::Visio);
    assert_eq!(relu.lieu.as_deref(), Some("Google Meet"));
    assert!(relu.analyse_ia.is_none());
}
