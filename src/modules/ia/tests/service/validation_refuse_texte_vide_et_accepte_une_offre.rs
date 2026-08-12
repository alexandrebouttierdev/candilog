//! Cas de test isolé.

use super::*;

#[test]
fn validation_refuse_texte_vide_et_accepte_une_offre() {
    assert!(validate_text("", "L'offre").is_err());
    assert!(validate_text("Administrateur Linux", "L'offre").is_ok());
}

#[test]
fn validation_n_impose_plus_de_plafond_de_caracteres() {
    let offre = "A".repeat(200_001);
    assert!(validate_text(&offre, "L'offre").is_ok());
}
