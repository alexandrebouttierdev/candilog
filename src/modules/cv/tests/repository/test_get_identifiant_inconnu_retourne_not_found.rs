//! Cas de test isolé.

use super::*;

#[test]
fn test_get_identifiant_inconnu_retourne_not_found() {
    let repo = repo();
    let resultat = repo.get(uuid::Uuid::new_v4());
    assert!(matches!(resultat, Err(AppError::NotFound(_))));
}
