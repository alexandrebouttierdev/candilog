//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_refuse_une_entreprise_vide() {
    let service = ApplicationService::new(StubRepo::default());
    let mut input = new("Développeur");
    input.company_id = uuid::Uuid::nil();

    let erreur = service.create(&input).unwrap_err();
    assert!(matches!(erreur, AppError::Validation(_)));
}
