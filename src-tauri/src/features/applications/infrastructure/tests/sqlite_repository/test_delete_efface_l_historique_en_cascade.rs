//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_efface_l_historique_en_cascade() {
    let (repo, company_id) = context();
    let creee = repo
        .create(&entree(company_id, "Développeur", "2026-08-20"))
        .unwrap();

    repo.delete(creee.id).unwrap();

    assert!(history(&repo, creee.id).is_empty());
    assert!(matches!(repo.get(creee.id), Err(AppError::NotFound(_))));
}
