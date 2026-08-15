//! Cas de test isolé.

use super::*;

#[test]
fn lister_delegue_au_depot() {
    let service = SecteurService::new(StubRepo);
    let secteurs = service.lister().unwrap();
    assert_eq!(secteurs.len(), 1);
    assert_eq!(secteurs[0].nom, "Santé");
}
