//! Cas de test isolé.

use super::*;

/// Les filtres de période comparent des chaînes (`c.sent_date >= ?`) : une date au format
/// `JJ-MM-AAAA` s'y comparerait dans le désordre et disparaîtrait silencieusement des
/// résultats, sans qu'aucune erreur ne le signale.
#[test]
fn test_date_dans_un_autre_format_est_refusee() {
    let service = ApplicationService::new(StubRepo);

    for date in ["20-08-2026", "2026/08/20", "20 août 2026", ""] {
        let mut input = new("Développeur");
        input.sent_date = date.into();
        assert!(
            matches!(service.create(&input), Err(AppError::Validation(_))),
            "la date « {date} » aurait dû être refusée"
        );
    }
}
