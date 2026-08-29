use super::*;
use crate::features::profile::domain::{
    Certification, Education, Experience, ImportProfileRequest, ImportResolution,
    ImportScalarDecision, ImportSkillDecision, Language, Project, Skill,
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
        skills: vec![Skill {
            name: "Rust".into(),
        }],
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

fn empty_request() -> ImportProfileRequest {
    ImportProfileRequest {
        identity: vec![],
        experiences: vec![],
        skills: vec![],
        education: vec![],
        languages: vec![],
        projects: vec![],
        certifications: vec![],
    }
}

#[test]
fn preview_import_ne_modifie_pas_le_profil() {
    let existing = Profile {
        identity: Identity {
            email: "camille@example.fr".into(),
            ..Identity::default()
        },
        skills: vec![Skill {
            name: "Rust".into(),
        }],
        ..Profile::default()
    };
    let repo = Memoire {
        profile: Mutex::new(Some(existing.clone())),
    };
    let service = ProfileService::new(repo);
    let extracted = Profile {
        identity: Identity {
            first_name: "Camille".into(),
            ..Identity::default()
        },
        skills: vec![Skill {
            name: "React".into(),
        }],
        ..Profile::default()
    };

    let preview = service.preview_import(&extracted).unwrap();
    let after = service.load().unwrap();

    assert_eq!(after.profile, existing);
    assert_eq!(preview.counts.identity, 1);
    assert_eq!(preview.counts.skills, 1);
}

#[test]
fn apply_import_ecrit_une_seule_fois() {
    let repo = Memoire::default();
    let service = ProfileService::new(repo);
    let mut request = empty_request();
    request.skills = vec![ImportSkillDecision {
        id: "skill-0".into(),
        selected: true,
        value: Skill {
            name: "TypeScript".into(),
        },
        existing_index: None,
        resolution: ImportResolution::AddAsNew,
    }];

    let result = service.apply_import(&request).unwrap();
    let loaded = service.load().unwrap();

    assert_eq!(result.added, 1);
    assert_eq!(loaded.profile.skills[0].name, "TypeScript");
}

#[test]
fn apply_import_ne_ecrit_pas_si_la_validation_echoue() {
    let existing = Profile {
        skills: vec![Skill {
            name: "Rust".into(),
        }],
        ..Profile::default()
    };
    let repo = Memoire {
        profile: Mutex::new(Some(existing.clone())),
    };
    let service = ProfileService::new(repo);
    let mut request = empty_request();
    request.identity = vec![ImportScalarDecision {
        id: "email".into(),
        selected: true,
        value: "pas-un-email".into(),
        resolution: ImportResolution::Replace,
    }];

    let error = service.apply_import(&request).unwrap_err();
    let after = service.load().unwrap();

    assert!(matches!(error, AppError::Validation(_)));
    assert_eq!(after.profile, existing);
}
