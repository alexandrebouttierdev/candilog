//! Cas de test isolé.

use super::*;

/// Le calendrier borne un mois entier : exclure les bornes ferait disparaître les
/// entretiens du premier et du dernier jour affichés.
#[test]
fn test_la_plage_du_calendrier_inclut_ses_bornes() {
    let (repo, application_id) = context();
    for date in [
        "2026-08-01T09:00:00+02:00",
        "2026-08-15T14:00:00+02:00",
        "2026-08-31T18:00:00+02:00",
    ] {
        repo.save_and_mark_candidate(None, &entree(application_id, date))
            .unwrap();
    }

    let in_month = repo
        .list_between("2026-08-01T00:00:00+00:00", "2026-08-31T23:59:59+00:00")
        .unwrap();

    assert_eq!(in_month.len(), 3);
}
