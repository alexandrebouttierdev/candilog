//! Cas de test isolé.

use super::*;

#[test]
fn test_absence_de_coordonnees_renvoie_vide() {
    let c = extract_contacts("Développeur passionné, curieux et rigoureux.");
    assert_eq!(c, Contacts::default());
}
