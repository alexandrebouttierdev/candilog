//! Cas de test isolé.

use super::*;

#[test]
fn test_enregistrer_sur_identifiant_inconnu_retourne_not_found() {
    let (repo, candidature_id) = contexte();

    let resultat = repo.save_and_mark_candidate(
        Some(Uuid::new_v4()),
        &entree(candidature_id, "2026-08-25T14:00:00+02:00"),
    );

    assert!(matches!(resultat, Err(AppError::NotFound(_))));
}
