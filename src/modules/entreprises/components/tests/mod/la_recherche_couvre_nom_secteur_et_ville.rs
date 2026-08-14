//! Cas de test isolé.

use super::*;

#[test]
fn la_recherche_couvre_nom_secteur_et_ville() {
    let company = entreprise(Some("Rennes"), Some("Agroalimentaire"));
    assert!(matches(&company, ""));
    assert!(matches(&company, "agrial"));
    assert!(matches(&company, "rennes"));
    assert!(matches(&company, "agroalim"));
    assert!(!matches(&company, "nantes"));
}
