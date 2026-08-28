//! Cas de test isolé.

use super::*;

#[test]
fn test_enum_depuis_texte_valeur_inconnue_retourne_erreur() {
    let resultat = enum_from_text::<StatusFactice>("N_IMPORTED_RESOURCE");
    assert!(matches!(resultat, Err(AppError::Serialization(_))));
}
