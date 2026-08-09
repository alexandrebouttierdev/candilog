//! Cas de test isolé.

use super::*;

#[test]
fn test_update_identifiant_inconnu_retourne_not_found() {
    let repo = repo();
    let ent = entreprise(&repo, "ACME");
    let resultat = repo.update(uuid::Uuid::new_v4(), &entree(ent, "Dev"));
    assert!(matches!(resultat, Err(AppError::NotFound(_))));
}
