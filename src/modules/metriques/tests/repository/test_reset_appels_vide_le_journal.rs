//! Cas de test isolé.

use super::*;

#[test]
fn test_reset_appels_vide_le_journal() {
    let r = repo();
    r.enregistrer_appel(&appel(
        OperationLlm::AnalyserEntretien,
        "2026-07-16T10:00:00Z",
        false,
    ))
    .unwrap();
    r.reset_appels().unwrap();
    assert!(r.lister_appels().unwrap().is_empty());
}
