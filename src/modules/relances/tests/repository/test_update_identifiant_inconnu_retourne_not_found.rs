//! Cas de test isolé.

use super::*;

#[test]
fn test_update_identifiant_inconnu_retourne_not_found() {
    let repo = repo();
    let cand = candidature(&repo);
    let resultat = repo.update(uuid::Uuid::new_v4(), &entree(cand, "2026-02-01"));
    assert!(matches!(resultat, Err(AppError::NotFound(_))));
}
