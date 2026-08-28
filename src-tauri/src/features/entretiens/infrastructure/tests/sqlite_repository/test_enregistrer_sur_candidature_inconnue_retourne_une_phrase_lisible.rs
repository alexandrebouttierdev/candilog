//! Cas de test isolé.

use super::*;

/// La violation de clé étrangère remonterait sinon en `SQLITE_CONSTRAINT` brut.
#[test]
fn test_enregistrer_sur_candidature_inconnue_retourne_une_phrase_lisible() {
    let (repo, _) = contexte();

    let resultat =
        repo.save_and_mark_candidate(None, &entree(Uuid::new_v4(), "2026-08-25T14:00:00+02:00"));

    match resultat {
        Err(AppError::Validation(message)) => assert!(message.contains("candidature")),
        autre => panic!("attendu Validation, obtenu {autre:?}"),
    }
}
