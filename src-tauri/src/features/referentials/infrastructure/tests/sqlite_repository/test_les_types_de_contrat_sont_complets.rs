//! Le catalogue des contrats est semé en entier, codes persistés et libellés français.

use super::*;

#[test]
fn les_vingt_deux_contrats_sont_presents_avec_leur_libelle() {
    let contracts = repo().load().unwrap().contract_types;

    assert_eq!(contracts.len(), 22);
    assert_eq!(label(&contracts, "CDI").as_deref(), Some("CDI"));
    assert_eq!(label(&contracts, "CDD").as_deref(), Some("CDD"));
    assert_eq!(label(&contracts, "MIS").as_deref(), Some("Intérim"));
    assert_eq!(
        label(&contracts, "E2").as_deref(),
        Some("Contrat apprentissage")
    );
    assert_eq!(
        label(&contracts, "FS").as_deref(),
        Some("Cont. professionnalisation")
    );
    assert_eq!(
        label(&contracts, "REP").as_deref(),
        Some("Reprise d'entreprise")
    );
}

#[test]
fn le_premier_contrat_propose_est_le_cdi() {
    let contracts = repo().load().unwrap().contract_types;

    assert_eq!(
        contracts.first().map(|item| item.code.as_str()),
        Some("CDI")
    );
}
