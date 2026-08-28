//! Cas de test isolé.

use super::*;

#[test]
fn test_enregistrer_valide_delegue_au_depot() {
    let service = InterviewService::new(StubRepo);
    let enregistre = service
        .save(None, &new("2026-08-25T14:00:00+02:00"))
        .unwrap();

    assert_eq!(enregistre.interview_date, "2026-08-25T14:00:00+02:00");
}
