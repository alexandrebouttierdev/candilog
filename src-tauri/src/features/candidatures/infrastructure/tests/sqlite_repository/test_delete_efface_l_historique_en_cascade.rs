//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_efface_l_historique_en_cascade() {
    let (repo, entreprise_id) = contexte();
    let creee = repo
        .create(&entree(entreprise_id, "Développeur", "2026-08-20"))
        .unwrap();

    repo.delete(creee.id).unwrap();

    assert!(historique(&repo, creee.id).is_empty());
    assert!(matches!(repo.get(creee.id), Err(AppError::NotFound(_))));
}
