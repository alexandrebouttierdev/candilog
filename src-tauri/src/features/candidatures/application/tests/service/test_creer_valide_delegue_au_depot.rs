//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_valide_delegue_au_depot() {
    let service = CandidatureService::new(StubRepo);
    let creee = service.creer(&nouvelle("Développeur Frontend")).unwrap();

    assert_eq!(creee.poste, "Développeur Frontend");
}
