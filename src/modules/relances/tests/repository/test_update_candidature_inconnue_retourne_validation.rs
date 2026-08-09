//! Cas de test isolé.

use super::*;

#[test]
fn test_update_candidature_inconnue_retourne_validation() {
    let repo = repo();
    let cand = candidature(&repo);
    let creee = repo.create(&entree(cand, "2026-02-01")).unwrap();
    let resultat = repo.update(creee.id, &entree(uuid::Uuid::new_v4(), "2026-02-15"));
    assert!(matches!(resultat, Err(AppError::Validation(_))));
}
