//! Cas de test isolé.

use super::*;

#[test]
fn les_contrats_ont_une_abreviation_courte() {
    for contract in [
        TypeContrat::Cdi,
        TypeContrat::Cdd,
        TypeContrat::Freelance,
        TypeContrat::Stage,
        TypeContrat::Alternance,
        TypeContrat::Interim,
        TypeContrat::Autre,
    ] {
        let short = contract_short(contract);
        assert!(!short.is_empty());
        assert!(short.chars().count() <= 10, "abréviation trop longue");
    }
}
