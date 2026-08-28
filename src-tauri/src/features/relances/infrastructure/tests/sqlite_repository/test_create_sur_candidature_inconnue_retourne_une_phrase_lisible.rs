//! Cas de test isolé.

use super::*;

#[test]
fn test_create_sur_candidature_inconnue_retourne_une_phrase_lisible() {
    let (repo, _) = contexte();

    let resultat = repo.create(&entree(Uuid::new_v4(), "2026-08-27"));

    match resultat {
        Err(AppError::Validation(message)) => assert!(message.contains("candidature")),
        autre => panic!("attendu Validation, obtenu {autre:?}"),
    }
}
