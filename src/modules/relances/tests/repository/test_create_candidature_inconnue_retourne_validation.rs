//! Cas de test isolé.

use super::*;

#[test]
fn test_create_candidature_inconnue_retourne_validation() {
    let repo = repo();
    let resultat = repo.create(&entree(uuid::Uuid::new_v4(), "2026-02-01"));
    assert!(matches!(resultat, Err(AppError::Validation(_))));
}
