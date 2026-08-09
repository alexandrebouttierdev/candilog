//! Cas de test isolé.

use super::*;

#[test]
fn test_update_entreprise_inconnue_retourne_validation() {
    let repo = repo();
    let ent = entreprise(&repo, "ACME");
    let creee = repo.create(&entree(ent, "Dev")).unwrap();
    let resultat = repo.update(creee.id, &entree(uuid::Uuid::new_v4(), "Dev"));
    assert!(matches!(resultat, Err(AppError::Validation(_))));
}
