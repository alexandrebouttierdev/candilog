//! Cas de test isolé.

use super::*;

#[test]
fn test_changer_statut_delegue_au_depot() {
    let svc = CandidatureService::new(MockRepo::default());
    svc.changer_statut(Uuid::nil(), StatutCandidature::Entretien)
        .unwrap();
    assert_eq!(
        svc.repo.statuts.lock().unwrap()[0],
        StatutCandidature::Entretien
    );
}
