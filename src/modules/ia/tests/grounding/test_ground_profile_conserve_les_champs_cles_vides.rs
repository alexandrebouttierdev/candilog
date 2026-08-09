//! Cas de test isolé.

use super::*;

#[test]
fn test_ground_profile_conserve_les_champs_cles_vides() {
    // Une expérience sans employeur (titre seul) n'est pas jugée sur `company`.
    let mut profile = Profile {
        experiences: vec![Experience {
            title: "Bénévole".into(),
            company: String::new(),
            ..Default::default()
        }],
        ..Default::default()
    };
    ground_profile(&mut profile, CV);
    assert_eq!(profile.experiences.len(), 1);
}
