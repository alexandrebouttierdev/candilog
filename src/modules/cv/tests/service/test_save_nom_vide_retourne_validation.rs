//! Cas de test isolé.

use super::*;

#[test]
fn test_save_nom_vide_retourne_validation() {
    let svc = CvVersionService::new(MockRepo::default());
    let r = svc.save("   ", &Value::Null);
    assert!(matches!(r, Err(AppError::Validation(_))));
}
