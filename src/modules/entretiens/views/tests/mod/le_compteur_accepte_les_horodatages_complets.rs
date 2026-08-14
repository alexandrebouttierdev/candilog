//! Cas de test isolé.

use super::*;

#[test]
fn le_compteur_accepte_les_horodatages_complets() {
    let entretiens = [entretien("2026-12-15T18:00:00+02:00")];
    let relances = [relance("2026-12-01T08:00:00+02:00")];
    let (total, interviews, reminders) = month_counts(&entretiens, &relances, 2026, 12);
    assert_eq!((total, interviews, reminders), (2, 1, 1));
}
