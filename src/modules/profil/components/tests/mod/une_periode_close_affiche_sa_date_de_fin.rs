//! Cas de test isolé.

use super::*;

#[test]
fn une_periode_close_affiche_sa_date_de_fin() {
    let experience = Experience {
        start_date: "2020-01".into(),
        end_date: Some("2022-12".into()),
        current: false,
        ..Experience::default()
    };
    assert_eq!(experience_period(&experience), "2020-01 → 2022-12");
}
