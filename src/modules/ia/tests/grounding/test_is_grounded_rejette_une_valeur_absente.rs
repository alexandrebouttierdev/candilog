//! Cas de test isolé.

use super::*;

#[test]
fn test_is_grounded_rejette_une_valeur_absente() {
    let src = fold(CV);
    assert!(!is_grounded(&src, "Google"));
    assert!(!is_grounded(&src, "Kubernetes"));
}
