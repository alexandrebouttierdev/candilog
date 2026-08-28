//! Cas de test isolé.

use super::*;

#[test]
fn test_get_identifiant_inconnu_retourne_not_found() {
    let (repo, _) = contexte();
    assert!(matches!(
        repo.get(Uuid::new_v4()),
        Err(AppError::NotFound(_))
    ));
}
