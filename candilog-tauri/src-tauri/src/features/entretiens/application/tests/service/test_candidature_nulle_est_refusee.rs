//! Cas de test isolé.

use super::*;

/// Un entretien sans candidature n'a pas de sens métier, et la clé étrangère `NOT NULL` le
/// refuserait de toute façon — mais avec un message technique.
#[test]
fn test_candidature_nulle_est_refusee() {
    let service = EntretienService::new(StubRepo);
    let mut input = nouvel("2026-08-25T14:00:00+02:00");
    input.candidature_id = uuid::Uuid::nil();

    assert!(matches!(
        service.enregistrer(None, &input),
        Err(AppError::Validation(_))
    ));
}
