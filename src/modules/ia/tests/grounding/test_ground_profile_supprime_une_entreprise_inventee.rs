//! Cas de test isolé.

use super::*;

#[test]
fn test_ground_profile_supprime_une_entreprise_inventee() {
    let mut profile = Profile {
        experiences: vec![
            Experience {
                title: "Dev".into(),
                company: "ACME Corporation".into(),
                ..Default::default()
            },
            Experience {
                title: "Dev".into(),
                company: "Google".into(),
                ..Default::default()
            },
        ],
        skills: vec![
            Skill {
                name: "Rust".into(),
            },
            Skill {
                name: "Kubernetes".into(),
            },
        ],
        certifications: vec![Certification {
            name: "AWS Solutions Architect".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    ground_profile(&mut profile, CV);
    assert_eq!(profile.experiences.len(), 1);
    assert_eq!(profile.experiences[0].company, "ACME Corporation");
    assert_eq!(profile.skills.len(), 1);
    assert_eq!(profile.skills[0].name, "Rust");
    assert_eq!(profile.certifications.len(), 1); // ancrée, conservée
}
