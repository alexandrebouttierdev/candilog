//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_valide_delegue_au_depot() {
    let service = ApplicationService::new(StubRepo);
    let creee = service.create(&new("Développeur Frontend")).unwrap();

    assert_eq!(creee.job_title, "Développeur Frontend");
}
