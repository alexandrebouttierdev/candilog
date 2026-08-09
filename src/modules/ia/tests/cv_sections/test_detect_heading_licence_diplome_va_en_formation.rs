//! Cas de test isolé.

use super::*;

#[test]
fn test_detect_heading_licence_diplome_va_en_formation() {
    // « licence » est exclu des certifications (ambigu avec le diplôme français) ;
    // un titre de diplôme n'est pas un en-tête de section.
    assert_eq!(detect_heading("Licence Informatique"), None);
}
