//! Cas de test isolé.

use super::*;

#[test]
fn fusionne_l_identite_et_le_contenu_genere() {
    let resume = build(&profile(), &generation());
    assert_eq!(resume.name, "Alex Exemple");
    assert_eq!(resume.subtitle, "Administrateur systèmes");
    assert_eq!(resume.profile, "Résumé généré.");
    assert_eq!(resume.skills, vec!["Linux"]);
}
