//! Cas de test isolé.

use super::*;

#[test]
fn test_score_experience_suffisante_donne_100() {
    let p = Profile {
        experiences: vec![Experience {
            title: "Dev".into(),
            company: "ACME".into(),
            start_date: "2018".into(),
            end_date: Some("2023".into()),
            current: false,
            ..Experience::default()
        }],
        ..Profile::default()
    };
    let o = offer(&[], &[], Some("3 ans"));
    assert_eq!(score(&p, &o).experience, 100);
}
