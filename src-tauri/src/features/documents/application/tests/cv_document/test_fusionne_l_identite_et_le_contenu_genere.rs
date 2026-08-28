//! Cas de test isolé.

use super::*;

#[test]
fn fusionne_l_identite_et_le_contenu_genere() {
    let cv = construire(&profil(), &generation());
    assert_eq!(cv.name, "Alex Exemple");
    assert_eq!(cv.subtitle, "Administrateur systèmes");
    assert_eq!(cv.profil, "Résumé généré.");
    assert_eq!(cv.skills, vec!["Linux"]);
}
