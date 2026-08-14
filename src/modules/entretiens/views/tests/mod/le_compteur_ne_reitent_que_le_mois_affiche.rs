//! Cas de test isolé.

use super::*;

#[test]
fn le_compteur_ne_reitent_que_le_mois_affiche() {
    let entretiens = [
        entretien("2026-08-05T10:00:00+02:00"),
        entretien("2026-08-20"),
        entretien("2026-07-31"),
        entretien("2026-09-01T14:30:00+02:00"),
    ];
    let relances = [
        relance("2026-08-09"),
        relance("2026-08-31T09:00:00+02:00"),
        relance("2026-08-01"),
        relance("2026-10-02"),
    ];
    let (total, interviews, reminders) = month_counts(&entretiens, &relances, 2026, 8);
    assert_eq!(total, 5);
    assert_eq!(interviews, 2);
    assert_eq!(reminders, 3);
}
