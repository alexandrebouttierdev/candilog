//! Cas de test isolé.

use super::*;

/// Un `None` doit produire une cellule vide, jamais la chaîne « None » : le fichier est lu
/// par un humain dans un tableur.
#[test]
fn test_les_champs_absents_deviennent_vides() {
    let mut sans_entreprise = cand("Développeur", None);
    sans_entreprise.entreprise_nom = None;
    sans_entreprise.entreprise_ville = None;

    let csv = vers_csv(&[sans_entreprise]).unwrap();

    assert!(csv.contains("Développeur;;;CDI"));
    assert!(!csv.contains("None"));
}
