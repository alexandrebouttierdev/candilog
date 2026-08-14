//! Cas de test isolé.

use super::*;

#[test]
fn le_nom_complet_est_normalise() {
    assert_eq!(full_name(&contact(None, None)), "Alex Bouttier");
}
