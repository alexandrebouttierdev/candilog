//! Cas de test isolé.

use super::*;

#[test]
fn test_enum_depuis_texte_valeur_connue_retourne_la_variante() {
    let statut: StatutCandidature = enum_depuis_texte("EN_ATTENTE").unwrap();
    assert_eq!(statut, StatutCandidature::EnAttente);
}
