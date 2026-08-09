//! Cas de test isolé.

use super::*;

#[test]
fn test_enregistrer_appel_puis_lister_restitue_les_champs() {
    let r = repo();
    r.enregistrer_appel(&appel(
        OperationLlm::ParseOffer,
        "2026-07-16T10:00:00Z",
        true,
    ))
    .unwrap();
    let appels = r.lister_appels().unwrap();
    assert_eq!(appels.len(), 1);
    assert_eq!(appels[0].operation, OperationLlm::ParseOffer);
    assert_eq!(appels[0].provider, "ollama");
    assert_eq!(appels[0].latence_ms, 120);
    assert!(appels[0].succes);
}
