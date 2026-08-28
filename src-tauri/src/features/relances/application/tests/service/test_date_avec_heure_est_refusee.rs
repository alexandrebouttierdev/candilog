//! Cas de test isolé.

use super::*;

/// Une relance se programme au jour, sans heure, contrairement à un entretien : un
/// horodatage complet ne se comparerait pas correctement aux bornes du calendrier, qui
/// sont des dates nues.
#[test]
fn test_date_avec_heure_est_refusee() {
    let service = RelanceService::new(StubRepo);

    for date in [
        "2026-08-27T10:00:00Z",
        "27-08-2026",
        "",
        "la semaine prochaine",
    ] {
        let mut input = nouvelle("2026-08-27");
        input.date_relance = date.into();
        assert!(
            matches!(service.creer(&input), Err(AppError::Validation(_))),
            "la date « {date} » aurait dû être refusée"
        );
    }
}
