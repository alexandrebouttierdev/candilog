//! Cas de test isolé.

use super::*;

#[test]
fn la_recherche_couvre_nom_fonction_et_courriel() {
    let contact = contact(Some("Responsable RH"), Some("alex@agrial.fr"));
    assert!(matches(&contact, ""));
    assert!(matches(&contact, "bouttier"));
    assert!(matches(&contact, "responsable"));
    assert!(matches(&contact, "agrial"));
    assert!(!matches(&contact, "dupont"));
}
