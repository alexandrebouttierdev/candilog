//! Cas de test isolé.

use super::*;

/// La colonne est `NOT NULL` avec une clé étrangère : sans ce contrôle, l'erreur ne
/// surviendrait qu'au niveau SQL, sous la forme d'une violation de contrainte illisible.
#[test]
fn test_creer_refuse_un_contrat_vide() {
    let service = ApplicationService::new(StubRepo::default());
    let mut input = new("Développeur");
    input.contract_type_code = "   ".into();

    assert!(matches!(
        service.create(&input),
        Err(AppError::Validation(_))
    ));
}
