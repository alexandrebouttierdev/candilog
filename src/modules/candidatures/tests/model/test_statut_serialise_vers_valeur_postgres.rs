//! Cas de test isolé.

use super::*;

#[test]
fn test_statut_serialise_vers_valeur_postgres() {
    assert_eq!(
        serde_json::to_string(&StatutCandidature::EnAttente).unwrap(),
        "\"EN_ATTENTE\""
    );
    assert_eq!(
        serde_json::to_string(&StatutCandidature::Relancee).unwrap(),
        "\"RELANCEE\""
    );
    assert_eq!(
        serde_json::to_string(&StatutCandidature::Refus).unwrap(),
        "\"REFUS\""
    );
}
