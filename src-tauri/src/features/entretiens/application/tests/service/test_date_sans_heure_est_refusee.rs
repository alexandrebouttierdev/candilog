//! Cas de test isolé.

use super::*;

/// Un entretien porte une heure, contrairement à une relance : les requêtes de plage du
/// calendrier comparent des chaînes `RFC 3339`, et une date nue s'y comparerait avant
/// toutes les heures du même jour, faisant disparaître l'entretien de sa propre journée.
#[test]
fn test_date_sans_heure_est_refusee() {
    let service = EntretienService::new(StubRepo);

    for date in ["2026-08-25", "25-08-2026 14:00", "", "demain"] {
        let mut input = nouvel(date);
        input.date_entretien = date.into();
        assert!(
            matches!(
                service.enregistrer(None, &input),
                Err(AppError::Validation(_))
            ),
            "la date « {date} » aurait dû être refusée"
        );
    }
}
