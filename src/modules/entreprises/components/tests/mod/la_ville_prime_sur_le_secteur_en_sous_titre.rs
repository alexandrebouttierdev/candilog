//! Cas de test isolé.

use super::*;

#[test]
fn la_ville_prime_sur_le_secteur_en_sous_titre() {
    assert_eq!(
        subtitle(&entreprise(Some("Rennes"), Some("Agro"))),
        "Rennes"
    );
}
