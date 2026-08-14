//! Cas de test isolé.

use super::*;

#[test]
fn les_huit_semaines_s_ordonnent_de_la_plus_ancienne_a_la_courante() {
    let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let candidates = vec![
        candidature("2026-08-10"),
        candidature("2026-08-03"),
        candidature("2026-07-20"),
    ];
    let counts = weekly_counts(&candidates, today);
    assert_eq!(counts[7], 1, "la semaine courante");
    assert_eq!(counts[6], 1, "la semaine précédente");
    assert_eq!(counts[4], 1, "il y a trois semaines");
    assert_eq!(counts.iter().sum::<usize>(), 3);
}
