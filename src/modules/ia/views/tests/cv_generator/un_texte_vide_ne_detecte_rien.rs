//! Cas de test isolé.

use super::*;

#[test]
fn un_texte_vide_ne_detecte_rien() {
    let companies = vec![entreprise("Acme Corp")];
    assert_eq!(detected_company("", &companies), None);
    assert_eq!(detected_company("   ", &companies), None);
}
