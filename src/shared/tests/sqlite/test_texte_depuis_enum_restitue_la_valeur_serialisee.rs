//! Cas de test isolé.

use super::*;

#[test]
fn test_texte_depuis_enum_restitue_la_valeur_serialisee() {
    assert_eq!(
        texte_depuis_enum(&StatutCandidature::EnAttente).unwrap(),
        "EN_ATTENTE"
    );
}
