//! Cas de test isolé.

use super::*;

#[test]
fn test_detect_heading_ignore_les_lignes_longues_et_le_corps() {
    assert_eq!(
        detect_heading("Expériences professionnelles"),
        Some(Bucket::Experience)
    );
    assert_eq!(detect_heading("FORMATION"), Some(Bucket::Education));
    // Ligne de corps trop longue malgré le mot-clé.
    assert_eq!(
        detect_heading("Grâce à mon expérience de dix ans dans le développement logiciel embarqué"),
        None,
    );
    assert_eq!(detect_heading("Ingénieure logiciel senior"), None);
}
