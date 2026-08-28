//! Cas de test isolé.

use super::*;

#[test]
fn test_creer_valide_delegue_au_depot() {
    let service = FollowUpService::new(StubRepo);
    let creee = service.create(&new("2026-08-27")).unwrap();

    assert_eq!(creee.follow_up_date, "2026-08-27");
    assert_eq!(creee.channel, "Email");
}
