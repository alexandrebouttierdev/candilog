//! Cas de test isolé.

use super::*;

#[test]
fn sans_entreprise_aucune_candidature_n_est_comptee() {
    assert_eq!(total_candidatures(&[], &[]), 0);
}
