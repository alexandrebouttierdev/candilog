//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_poste_vide_retourne_validation() {
    let service = ApplicationService::new(StubRepo::default());
    let mut input = new("Développeur");
    input.job_title = "   ".into();

    assert!(matches!(
        service.create(&input),
        Err(AppError::Validation(_))
    ));
}
