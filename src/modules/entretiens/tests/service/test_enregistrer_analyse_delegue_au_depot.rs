//! Cas de test isolé.

use super::*;

#[test]
fn test_enregistrer_analyse_delegue_au_depot() {
    let svc = EntretienService::new(StubRepo);
    let analyse = AnalyseEntretien {
        resume: "ok".into(),
        ..Default::default()
    };
    assert!(svc.enregistrer_analyse(uuid::Uuid::nil(), &analyse).is_ok());
}
