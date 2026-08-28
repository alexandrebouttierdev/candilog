//! Cas de test isolé.

use super::*;

#[test]
fn pose_l_identite_et_l_objet() {
    let pdf = build_cover_letter(&profile(), &cover_letter());
    assert_eq!(pdf.name, "Alex Exemple");
    assert_eq!(pdf.email, "alex@exemple.fr");
    assert!(pdf.subject.contains("Développeur"));
    assert!(pdf.subject.contains("Nova"));
}
