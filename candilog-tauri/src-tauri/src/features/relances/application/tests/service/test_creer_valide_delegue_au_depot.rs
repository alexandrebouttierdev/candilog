//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_valide_delegue_au_depot() {
    let service = RelanceService::new(StubRepo);
    let creee = service.creer(&nouvelle("2026-08-27")).unwrap();

    assert_eq!(creee.date_relance, "2026-08-27");
    assert_eq!(creee.type_relance, "Email");
}
