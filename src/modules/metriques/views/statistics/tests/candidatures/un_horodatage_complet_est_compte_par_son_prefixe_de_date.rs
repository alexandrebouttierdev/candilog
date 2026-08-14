//! Cas de test isolé.

use super::*;

#[test]
fn un_horodatage_complet_est_compte_par_son_prefixe_de_date() {
    let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let candidates = vec![candidature("2026-08-10T14:30:00Z")];
    let counts = weekly_counts(&candidates, today);
    assert_eq!(counts[7], 1);
}
