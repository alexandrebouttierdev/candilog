//! Cas de test isolé.

use super::*;

/// Le calendrier borne un mois entier : exclure les bornes ferait disparaître les
/// entretiens du premier et du dernier jour affichés.
#[test]
fn test_la_plage_du_calendrier_inclut_ses_bornes() {
    let (repo, candidature_id) = contexte();
    for date in [
        "2026-08-01T09:00:00+02:00",
        "2026-08-15T14:00:00+02:00",
        "2026-08-31T18:00:00+02:00",
    ] {
        repo.save_and_mark_candidate(None, &entree(candidature_id, date))
            .unwrap();
    }

    let dans_le_mois = repo
        .list_between("2026-08-01T00:00:00+00:00", "2026-08-31T23:59:59+00:00")
        .unwrap();

    assert_eq!(dans_le_mois.len(), 3);
}
