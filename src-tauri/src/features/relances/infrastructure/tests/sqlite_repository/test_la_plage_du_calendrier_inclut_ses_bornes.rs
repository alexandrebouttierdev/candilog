//! Cas de test isolé.

use super::*;

/// Le calendrier borne un mois entier : exclure les bornes ferait disparaître les relances
/// du premier et du dernier jour affichés.
#[test]
fn test_la_plage_du_calendrier_inclut_ses_bornes() {
    let (repo, candidature_id) = contexte();
    for date in ["2026-08-01", "2026-08-15", "2026-08-31"] {
        repo.create(&entree(candidature_id, date)).unwrap();
    }

    assert_eq!(
        repo.list_between("2026-08-01", "2026-08-31").unwrap().len(),
        3
    );
    assert_eq!(
        repo.list_between("2026-08-02", "2026-08-30").unwrap().len(),
        1
    );
}
