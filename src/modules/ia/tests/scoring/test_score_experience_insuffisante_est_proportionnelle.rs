//! Cas de test isolé.

use super::*;

#[test]
fn test_score_experience_insuffisante_est_proportionnelle() {
    let p = Profile {
        experiences: vec![Experience {
            title: "Dev".into(),
            company: "ACME".into(),
            start_date: "2022".into(),
            end_date: Some("2023".into()),
            current: false,
            ..Experience::default()
        }],
        ..Profile::default()
    };
    let o = offer(&[], &[], Some("5 ans"));
    assert_eq!(score(&p, &o).experience, 20);
}
