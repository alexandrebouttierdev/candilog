//! Cas de test isolé.

use super::*;

#[test]
fn test_traduire_erreur_violation_de_cle_etrangere_retourne_validation() {
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
    let traduite = traduire_erreur(erreur, "entreprise liée introuvable");
    assert!(matches!(traduite, AppError::Validation(message) if message.contains("entreprise")));
}
