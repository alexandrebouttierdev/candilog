//! Cas de test isolé.

use super::*;

#[test]
fn test_save_nom_trop_long_retourne_validation() {
    let svc = CvVersionService::new(MockRepo::default());
    let long = "x".repeat(121);
    let r = svc.save(&long, &Value::Null);
    assert!(matches!(r, Err(AppError::Validation(_))));
}
