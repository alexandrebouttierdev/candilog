//! Cas de test isolé.

use super::*;

#[test]
fn test_update_identifiant_inconnu_retourne_not_found() {
    let repo = repo();
    let resultat = repo.update(uuid::Uuid::new_v4(), &entree("X", None));
    assert!(matches!(resultat, Err(AppError::NotFound(_))));
}
