//! Cas de test isolé.

use super::*;

#[test]
fn test_type_contrat_respecte_la_casse_postgres() {
    assert_eq!(
        serde_json::to_string(&TypeContrat::Freelance).unwrap(),
        "\"Freelance\""
    );
    assert_eq!(
        serde_json::to_string(&TypeContrat::Interim).unwrap(),
        "\"Interim\""
    );
    let t: TypeContrat = serde_json::from_str("\"CDI\"").unwrap();
    assert_eq!(t, TypeContrat::Cdi);
    let a: TypeContrat = serde_json::from_str("\"Autre\"").unwrap();
    assert_eq!(a, TypeContrat::Autre);
}
