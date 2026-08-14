//! Cas de test isolé.

use super::*;

#[test]
fn le_secteur_prend_le_relais_sans_ville() {
    assert_eq!(subtitle(&entreprise(None, Some("Agro"))), "Agro");
    assert_eq!(
        subtitle(&entreprise(Some("   "), None)),
        "Aucune localisation"
    );
}
