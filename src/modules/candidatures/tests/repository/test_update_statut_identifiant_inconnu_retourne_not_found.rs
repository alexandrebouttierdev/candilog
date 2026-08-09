//! Cas de test isolé.

use super::*;

#[test]
fn test_update_statut_identifiant_inconnu_retourne_not_found() {
    let repo = repo();
    let resultat = repo.update_statut(uuid::Uuid::new_v4(), StatutCandidature::Refus);
    assert!(matches!(resultat, Err(AppError::NotFound(_))));
}
