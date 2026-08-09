//! Cas de test isolé.

use super::*;

#[test]
fn test_lister_appels_ordonne_du_plus_recent_au_plus_ancien() {
    let r = repo();
    r.enregistrer_appel(&appel(
        OperationLlm::ParseOffer,
        "2026-07-16T10:00:00Z",
        true,
    ))
    .unwrap();
    r.enregistrer_appel(&appel(
        OperationLlm::GenerateCv,
        "2026-07-16T12:00:00Z",
        true,
    ))
    .unwrap();
    let ops: Vec<OperationLlm> = r
        .lister_appels()
        .unwrap()
        .into_iter()
        .map(|a| a.operation)
        .collect();
    assert_eq!(
        ops,
        vec![OperationLlm::GenerateCv, OperationLlm::ParseOffer]
    );
}
