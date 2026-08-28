//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_poste_vide_retourne_validation() {
    let service = CandidatureService::new(StubRepo);
    let mut input = nouvelle("Développeur");
    input.poste = "   ".into();

    assert!(matches!(
        service.creer(&input),
        Err(AppError::Validation(_))
    ));
}
