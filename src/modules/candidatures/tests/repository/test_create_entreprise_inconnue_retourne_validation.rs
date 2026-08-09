//! Cas de test isolé.

use super::*;

#[test]
fn test_create_entreprise_inconnue_retourne_validation() {
    let repo = repo();
    let resultat = repo.create(&entree(uuid::Uuid::new_v4(), "Dev"));
    assert!(matches!(resultat, Err(AppError::Validation(_))));
}
