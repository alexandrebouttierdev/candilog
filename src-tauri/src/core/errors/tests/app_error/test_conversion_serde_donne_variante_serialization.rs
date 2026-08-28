//! Cas de test isolé.

use super::*;

#[test]
fn test_conversion_serde_donne_variante_serialization() {
    let json_err = serde_json::from_str::<i32>("pas un nombre").unwrap_err();
    let err: AppError = json_err.into();
    assert!(matches!(err, AppError::Serialization(_)));
}
