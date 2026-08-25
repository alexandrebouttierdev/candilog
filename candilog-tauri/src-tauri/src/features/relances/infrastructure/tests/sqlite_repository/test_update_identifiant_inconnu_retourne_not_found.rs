//! Cas de test isolé.

use super::*;

#[test]
fn test_update_identifiant_inconnu_retourne_not_found() {
    let (repo, candidature_id) = contexte();

    let resultat = repo.update(Uuid::new_v4(), &entree(candidature_id, "2026-08-27"));

    assert!(matches!(resultat, Err(AppError::NotFound(_))));
}
