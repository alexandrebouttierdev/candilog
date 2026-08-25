//! Cas de test isolé.

use super::*;

#[test]
fn test_traduire_contrainte_violation_retourne_la_phrase_destinee_a_l_utilisateur() {
    let pool = crate::core::database::open_pool(None).unwrap();
    crate::core::database::run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    let erreur = conn
            .execute(
                "INSERT INTO candidatures (id, entreprise_id, poste, date_envoi, created_at, updated_at)
                 VALUES ('c1', 'inexistante', 'Dev', '2026-01-01', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                [],
            )
            .unwrap_err();
    let traduite =
        traduire_contrainte(erreur, "Phrase complète pour l'utilisateur.", "candidature");
    match traduite {
        AppError::Validation(message) => {
            assert_eq!(message, "Phrase complète pour l'utilisateur.");
        }
        autre => panic!("attendu Validation, obtenu {autre:?}"),
    }
}
