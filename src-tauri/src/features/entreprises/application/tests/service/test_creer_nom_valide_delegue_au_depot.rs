//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_nom_valide_delegue_au_depot() {
    let svc = EntrepriseService::new(StubRepo);
    let e = svc.creer(&nouvelle("ACME")).unwrap();
    assert_eq!(e.nom, "ACME");
}
