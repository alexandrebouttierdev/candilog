//! Cas de test isolé.

use super::*;

#[test]
fn test_enum_depuis_texte_valeur_inconnue_retourne_erreur() {
    let resultat = enum_depuis_texte::<StatutFactice>("N_IMPORTE_QUOI");
    assert!(matches!(resultat, Err(AppError::Serialization(_))));
}
