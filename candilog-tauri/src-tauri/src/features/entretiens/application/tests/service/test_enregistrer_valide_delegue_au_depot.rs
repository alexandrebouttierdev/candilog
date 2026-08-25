//! Cas de test isolé.

use super::*;

#[test]
fn test_enregistrer_valide_delegue_au_depot() {
    let service = EntretienService::new(StubRepo);
    let enregistre = service
        .enregistrer(None, &nouvel("2026-08-25T14:00:00+02:00"))
        .unwrap();

    assert_eq!(enregistre.date_entretien, "2026-08-25T14:00:00+02:00");
}
