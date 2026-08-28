//! Cas de test isolé.

use super::*;

#[test]
fn test_enum_depuis_texte_valeur_connue_retourne_la_variante() {
    let statut: StatutFactice = enum_depuis_texte("EN_ATTENTE").unwrap();
    assert_eq!(statut, StatutFactice::EnAttente);
}
