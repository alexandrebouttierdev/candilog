//! Cas de test isolé.

use super::*;

#[test]
fn test_statut_deserialise_depuis_valeur_postgres() {
    let s: StatutCandidature = serde_json::from_str("\"ENTRETIEN\"").unwrap();
    assert_eq!(s, StatutCandidature::Entretien);
}
