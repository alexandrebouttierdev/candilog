//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_poste_valide_delegue_au_depot() {
    let svc = CandidatureService::new(MockRepo::default());
    let c = svc.creer(&input("Dev Rust")).unwrap();
    assert_eq!(c.poste, "Dev Rust");
    assert_eq!(svc.repo.created.lock().unwrap()[0], "Dev Rust");
}
