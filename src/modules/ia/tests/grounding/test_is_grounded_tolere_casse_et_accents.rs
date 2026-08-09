//! Cas de test isolé.

use super::*;

#[test]
fn test_is_grounded_tolere_casse_et_accents() {
    let src = fold(CV);
    assert!(is_grounded(&src, "acme corporation"));
    assert!(is_grounded(&src, "RUST"));
    assert!(is_grounded(&src, "ingenieure")); // accent replié
}
