//! Cas de test isolé.

use super::*;

/// La violation de clé étrangère remonte sinon en `SQLITE_CONSTRAINT` brut, incompréhensible
/// à l'écran.
#[test]
fn test_create_sur_entreprise_inconnue_retourne_une_phrase_lisible() {
    let (repo, _) = contexte();

    let resultat = repo.create(&entree(Uuid::new_v4(), "Développeur", "2026-08-20"));

    match resultat {
        Err(AppError::Validation(message)) => assert!(message.contains("entreprise")),
        autre => panic!("attendu Validation, obtenu {autre:?}"),
    }
}
