use super::*;
use crate::features::profile::domain::{
    Certification, Skill, Experience, Education, Language, Project,
};
use std::sync::Mutex;

#[derive(Default)]
struct Memoire {
    profile: Mutex<Option<Profile>>,
}

impl ProfileRepository for Memoire {
    fn get(&self) -> AppResult<(Profile, Option<String>)> {
        Ok((
            self.profile.lock().unwrap().clone().unwrap_or_default(),
            None,
        ))
    }

    fn save(&self, profile: &Profile) -> AppResult<(Profile, String)> {
        *self.profile.lock().unwrap() = Some(profile.clone());
        Ok((profile.clone(), "2026-08-28T10:00:00Z".into()))
    }
}

#[test]
fn profil_vide_a_un_score_nul_et_sept_pistes() {
    let service = ProfileService::new(Memoire::default());

    let payload = service.load().unwrap();

    assert_eq!(payload.completion, 0);
    assert_eq!(payload.incomplete_sections.len(), 7);
}

#[test]
fn profil_complet_atteint_cent() {
    let profile = Profile {
        identity: Identity {
            first_name: "Camille".into(),
            name: "Rivet".into(),
            email: "camille@example.fr".into(),
            ..Identity::default()
        },
        experiences: vec![Experience {
            title: "Développeuse".into(),
            company: "Nova".into(),
            start_date: "2024-01".into(),
            current: true,
            ..Experience::default()
        }],
        skills: vec![Skill { name: "Rust".into() }],
        education: vec![Education {
            degree: "Master".into(),
            school: "Université".into(),
            ..Education::default()
        }],
        languages: vec![Language {
            name: "Français".into(),
            level: "Natif".into(),
        }],
        projects: vec![Project {
            name: "Candilog".into(),
            ..Project::default()
        }],
        certifications: vec![Certification {
            name: "AWS".into(),
            ..Certification::default()
        }],
    };
    let service = ProfileService::new(Memoire::default());

    let payload = service.save(&profile).unwrap();

    assert_eq!(payload.completion, 100);
    assert!(payload.incomplete_sections.is_empty());
}

#[test]
fn une_experience_sans_debut_est_refusee() {
    let profile = Profile {
        experiences: vec![Experience {
            title: "Développeuse".into(),
            company: "Nova".into(),
            ..Experience::default()
        }],
        ..Profile::default()
    };

    let error = ProfileService::new(Memoire::default())
        .save(&profile)
        .unwrap_err();

    assert!(matches!(error, AppError::Validation(_)));
}

#[test]
fn une_identite_avec_email_invalide_est_refusee() {
    let profile = Profile {
        identity: Identity {
            email: "camille@localhost".into(),
            ..Identity::default()
        },
        ..Profile::default()
    };

    let error = ProfileService::new(Memoire::default())
        .save(&profile)
        .unwrap_err();

    assert!(matches!(error, AppError::Validation(_)));
}

#[test]
fn une_entree_legacy_incomplete_ne_gonfle_pas_le_score() {
    let repo = Memoire {
        profile: Mutex::new(Some(Profile {
            experiences: vec![Experience {
                title: "Développeuse".into(),
                ..Experience::default()
            }],
            skills: vec![Skill::default()],
            ..Profile::default()
        })),
    };

    let payload = ProfileService::new(repo).load().unwrap();

    assert_eq!(payload.completion, 0);
    assert!(payload
        .incomplete_sections
        .contains(&"une expérience".into()));
    assert!(payload
        .incomplete_sections
        .contains(&"vos compétences".into()));
}
