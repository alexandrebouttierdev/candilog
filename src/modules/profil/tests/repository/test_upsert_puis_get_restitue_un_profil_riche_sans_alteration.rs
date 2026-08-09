//! Cas de test isolé.

use super::*;

#[test]
fn test_upsert_puis_get_restitue_un_profil_riche_sans_alteration() {
    use crate::shared::profile::{Experience, PersonalInfo, Skill};

    let repo = repo();
    let profil = Profile {
        personal: PersonalInfo {
            first_name: "Béatrice".into(),
            last_name: "Éloïse".into(),
            email: "beatrice@example.co".into(),
            city: Some("Montréal".into()),
            ..PersonalInfo::default()
        },
        experiences: vec![Experience {
            title: "Développeuse".into(),
            company: "Société Générale".into(),
            ..Experience::default()
        }],
        skills: vec![Skill {
            name: "Résilience".into(),
        }],
        ..Profile::default()
    };
    repo.upsert(&profil).unwrap();
    let relu = repo.get().unwrap();
    assert_eq!(relu, profil);
}
