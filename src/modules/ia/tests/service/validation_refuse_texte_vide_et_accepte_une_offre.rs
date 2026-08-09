//! Cas de test isolé.

use super::*;

#[test]
fn validation_refuse_texte_vide_et_accepte_une_offre() {
    assert!(validate_text("", "L'offre").is_err());
    assert!(validate_text("Administrateur Linux", "L'offre").is_ok());
}
