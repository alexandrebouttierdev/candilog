//! Cas de test isolé.

use super::*;

#[test]
fn test_extract_text_bytes_invalides_retourne_validation() {
    let r = extract_text(b"ceci n'est pas un pdf");
    assert!(matches!(r, Err(AppError::Validation(_))));
}
