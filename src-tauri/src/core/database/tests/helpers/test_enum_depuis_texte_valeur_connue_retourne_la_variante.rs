//! Cas de test isolé.

use super::*;

#[test]
fn test_enum_depuis_texte_valeur_connue_retourne_la_variante() {
    let status: StatusFactice = enum_from_text("EN_ATTENTE").unwrap();
    assert_eq!(status, StatusFactice::Pending);
}
