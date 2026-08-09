//! Cas de test isolé.

use super::*;

#[test]
fn test_normalise_les_niveaux_de_langue() {
    assert_eq!(
        normalize_language_level("Langue maternelle"),
        "Bilingue / natif"
    );
    assert_eq!(normalize_language_level("C1"), "Courant");
    assert_eq!(
        normalize_language_level("Niveau intermédiaire (B1)"),
        "Intermédiaire"
    );
    assert_eq!(normalize_language_level("notions"), "Débutant");
    assert_eq!(normalize_language_level("TOEIC 900"), "TOEIC 900");
}
