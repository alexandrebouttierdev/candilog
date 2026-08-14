//! Cas de test isolé.

use super::*;

#[test]
fn la_semaine_commence_toujours_un_lundi() {
    for day in 1..=28 {
        let date = NaiveDate::from_ymd_opt(2026, 8, day).expect("date valide");
        assert_eq!(week_start(date).weekday().num_days_from_monday(), 0);
        assert!(week_start(date) <= date);
    }
}
