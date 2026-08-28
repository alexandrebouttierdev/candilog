//! Cas de test isolé.

use super::*;

/// Les filtres de période comparent des chaînes (`c.date_envoi >= ?`) : une date au format
/// `JJ-MM-AAAA` s'y comparerait dans le désordre et disparaîtrait silencieusement des
/// résultats, sans qu'aucune erreur ne le signale.
#[test]
fn test_date_dans_un_autre_format_est_refusee() {
    let service = CandidatureService::new(StubRepo);

    for date in ["20-08-2026", "2026/08/20", "20 août 2026", ""] {
        let mut input = nouvelle("Développeur");
        input.date_envoi = date.into();
        assert!(
            matches!(service.creer(&input), Err(AppError::Validation(_))),
            "la date « {date} » aurait dû être refusée"
        );
    }
}
