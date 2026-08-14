//! Cas de test isolé.

use super::*;

#[test]
fn les_bornes_de_la_fenetre_des_huit_semaines_sont_respectees() {
    let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let candidates = vec![
        candidature("2026-08-09"),
        candidature("2026-06-16"), // 55 jours : dernière semaine incluse
        candidature("2026-06-15"), // 56 jours : hors fenêtre
        candidature("2026-08-11"), // future : ignorée
        candidature("pas-une-date"),
    ];
    let counts = weekly_counts(&candidates, today);
    assert_eq!(counts[7], 1);
    assert_eq!(counts[0], 1);
    assert_eq!(counts.iter().sum::<usize>(), 2);
}
