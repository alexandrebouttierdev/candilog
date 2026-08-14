//! Cas de test isolé.

use super::*;

#[test]
fn un_poste_en_cours_est_signale_explicitement() {
    let experience = Experience {
        start_date: "2023-06".into(),
        current: true,
        ..Experience::default()
    };
    assert_eq!(experience_period(&experience), "2023-06 → aujourd'hui");
}
