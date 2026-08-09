//! Cas de test isolé.

use super::*;

#[test]
fn test_delai_generation_accepte_les_modeles_locaux_lents() {
    assert_eq!(PROVIDER_GENERATION_TIMEOUT.as_secs(), 1_800);
}
