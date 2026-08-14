//! Cas de test isolé.

use super::*;

#[test]
fn aucune_entreprise_connue_ne_detecte_rien() {
    let companies = vec![entreprise("Acme Corp")];
    assert_eq!(
        detected_company("Poste au sein d'une startup inconnue.", &companies),
        None
    );
}
