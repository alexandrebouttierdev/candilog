//! Cas de test isolé.

use super::*;

#[test]
fn la_detection_ne_reconnait_pas_un_nom_court_dans_un_mot() {
    let companies = vec![entreprise("Dev"), entreprise("Globex")];
    assert_eq!(
        detected_company("Développeur backend chez Globex.", &companies),
        Some("Globex".to_owned())
    );
}
