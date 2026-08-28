//! Cas de test isolé.

use super::*;

#[test]
fn pose_l_identite_et_l_objet() {
    let pdf = construire_lettre(&profil(), &lettre());
    assert_eq!(pdf.nom, "Alex Exemple");
    assert_eq!(pdf.email, "alex@exemple.fr");
    assert!(pdf.objet.contains("Développeur"));
    assert!(pdf.objet.contains("Nova"));
}
