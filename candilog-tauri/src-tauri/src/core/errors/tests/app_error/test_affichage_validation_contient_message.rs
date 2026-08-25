//! Cas de test isolé.

use super::*;

#[test]
fn test_affichage_validation_contient_message() {
    let err = AppError::Validation("champ vide".into());
    assert!(err.to_string().contains("champ vide"));
}
