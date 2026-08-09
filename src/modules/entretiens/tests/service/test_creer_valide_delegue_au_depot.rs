//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_valide_delegue_au_depot() {
    let svc = EntretienService::new(StubRepo);
    assert_eq!(
        svc.creer(&nouveau(1, "2026-07-20T09:00:00Z"))
            .unwrap()
            .type_entretien,
        TypeEntretien::Presentiel
    );
}
