//! Cas de test isolé.

use super::*;

#[test]
fn une_periode_incomplete_reste_lisible() {
    let experience = Experience {
        start_date: "2019".into(),
        end_date: None,
        current: false,
        ..Experience::default()
    };
    assert_eq!(experience_period(&experience), "2019 → —");
}
