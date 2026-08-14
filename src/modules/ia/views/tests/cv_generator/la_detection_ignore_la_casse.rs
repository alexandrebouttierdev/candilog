//! Cas de test isolé.

use super::*;

#[test]
fn la_detection_ignore_la_casse() {
    let companies = vec![entreprise("Acme Corp")];
    assert_eq!(
        detected_company("Stage développeur chez ACME CORP.", &companies),
        Some("Acme Corp".to_owned())
    );
}
