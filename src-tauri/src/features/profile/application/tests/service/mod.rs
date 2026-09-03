use super::*;
use crate::features::profile::domain::{
    Certification, Education, Experience, ImportProfileRequest, ImportResolution,
    ImportScalarDecision, ImportSkillDecision, Language, Project, Skill, MAX_SIDE,
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

/// Service de test.
///
/// Le dossier de photos n'est créé qu'à la première écriture : les cas qui ne touchent pas
/// à la photo ne posent donc aucun fichier sur le disque. Ceux qui y touchent passent un
/// `tempfile::TempDir` explicite à `ProfileService::new`.
fn service<R: ProfileRepository>(repo: R) -> ProfileService<R> {
    ProfileService::new(repo, std::env::temp_dir().join("candilog-photos-tests"))
}

#[test]
fn profil_vide_a_un_score_nul_et_sept_pistes() {
    let service = service(Memoire::default());

    let payload = service.load().unwrap();

    assert_eq!(payload.completion, 0);
    assert_eq!(payload.incomplete_sections.len(), 7);
}

#[test]
fn profil_complet_atteint_cent() {
    let profile = Profile {
        photo: None,
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
    let service = service(Memoire::default());

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

    let error = service(Memoire::default()).save(&profile).unwrap_err();

    assert!(matches!(error, AppError::Validation(_)));
}

#[test]
fn une_identite_avec_email_invalide_est_refusee() {
    let profile = Profile {
        photo: None,
        identity: Identity {
            email: "camille@localhost".into(),
            ..Identity::default()
        },
        ..Profile::default()
    };

    let error = service(Memoire::default()).save(&profile).unwrap_err();

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

    let payload = service(repo).load().unwrap();

    assert_eq!(payload.completion, 0);
    assert!(payload
        .incomplete_sections
        .contains(&"une expérience".into()));
    assert!(payload
        .incomplete_sections
        .contains(&"vos compétences".into()));
}

fn service_avec_competences(noms: Vec<&str>) -> ProfileService<Memoire> {
    let profile = Profile {
        skills: noms
            .into_iter()
            .map(|name| Skill { name: name.into() })
            .collect(),
        ..Profile::default()
    };
    service(Memoire {
        profile: Mutex::new(Some(profile)),
    })
}

#[test]
fn ajoute_une_competence_sans_doublon_normalise() {
    let service = service_avec_competences(vec!["Café"]);
    service.add_skill(" cafe ").unwrap();
    assert_eq!(service.load().unwrap().profile.skills.len(), 1);
}

#[test]
fn ajoute_une_nouvelle_competence_distincte() {
    let service = service_avec_competences(vec!["Rust"]);
    service.add_skill("TypeScript").unwrap();
    let skills = service.load().unwrap().profile.skills;
    assert_eq!(skills.len(), 2);
    assert_eq!(skills[1].name, "TypeScript");
}

#[test]
fn refuse_une_competence_vide() {
    let service = service_avec_competences(vec![]);
    let error = service.add_skill("   ").unwrap_err();
    assert!(matches!(error, AppError::Validation(_)));
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
    let service = service(repo);
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
    let service = service(repo);
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
    let service = service(repo);
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

/// PNG 2×2 opaque, plus petit fichier valide utile aux tests de photo.
fn png_de_test() -> Vec<u8> {
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 255]))
        .write_to(&mut buffer, image::ImageFormat::Png)
        .unwrap();
    buffer.into_inner()
}

#[test]
fn enregistrer_une_photo_la_reference_dans_le_profil_et_ecrit_le_fichier() {
    let dossier = tempfile::tempdir().unwrap();
    let source = dossier.path().join("portrait.png");
    std::fs::write(&source, png_de_test()).unwrap();
    let photos = tempfile::tempdir().unwrap();
    let service = ProfileService::new(Memoire::default(), photos.path().to_path_buf());

    let payload = service.set_photo(&source).unwrap();

    let nom = payload.profile.photo.clone().expect("photo référencée");
    assert!(nom.ends_with(".png"));
    assert!(photos.path().join(&nom).is_file());
    assert!(service
        .photo_data_url()
        .unwrap()
        .unwrap()
        .starts_with("data:image/png;base64,"));
}

#[test]
fn remplacer_la_photo_supprime_l_ancien_fichier() {
    let dossier = tempfile::tempdir().unwrap();
    let source = dossier.path().join("portrait.png");
    std::fs::write(&source, png_de_test()).unwrap();
    let photos = tempfile::tempdir().unwrap();
    let service = ProfileService::new(Memoire::default(), photos.path().to_path_buf());

    let premiere = service.set_photo(&source).unwrap().profile.photo.unwrap();
    let seconde = service.set_photo(&source).unwrap().profile.photo.unwrap();

    assert_ne!(premiere, seconde);
    assert!(!photos.path().join(&premiere).exists());
    assert!(photos.path().join(&seconde).is_file());
}

struct EchecSauvegarde;

impl ProfileRepository for EchecSauvegarde {
    fn get(&self) -> AppResult<(Profile, Option<String>)> {
        Ok((Profile::default(), None))
    }

    fn save(&self, _profile: &Profile) -> AppResult<(Profile, String)> {
        Err(AppError::Database("écriture refusée".into()))
    }
}

#[test]
fn un_echec_de_sauvegarde_ne_laisse_pas_de_nouvelle_photo_orpheline() {
    let dossier = tempfile::tempdir().unwrap();
    let source = dossier.path().join("portrait.png");
    std::fs::write(&source, png_de_test()).unwrap();
    let photos = tempfile::tempdir().unwrap();
    let service = ProfileService::new(EchecSauvegarde, photos.path().to_path_buf());

    let error = service.set_photo(&source).unwrap_err();

    assert!(matches!(error, AppError::Database(_)));
    assert_eq!(std::fs::read_dir(photos.path()).unwrap().count(), 0);
}

#[test]
fn supprimer_la_photo_efface_la_reference_et_le_fichier() {
    let dossier = tempfile::tempdir().unwrap();
    let source = dossier.path().join("portrait.png");
    std::fs::write(&source, png_de_test()).unwrap();
    let photos = tempfile::tempdir().unwrap();
    let service = ProfileService::new(Memoire::default(), photos.path().to_path_buf());
    let nom = service.set_photo(&source).unwrap().profile.photo.unwrap();

    let payload = service.remove_photo().unwrap();

    assert_eq!(payload.profile.photo, None);
    assert!(!photos.path().join(&nom).exists());
    assert_eq!(service.photo_data_url().unwrap(), None);
}

#[test]
fn un_fichier_qui_n_est_pas_une_image_est_refuse() {
    let dossier = tempfile::tempdir().unwrap();
    let source = dossier.path().join("faux.png");
    std::fs::write(&source, b"ceci n'est pas une image").unwrap();
    let photos = tempfile::tempdir().unwrap();
    let service = ProfileService::new(Memoire::default(), photos.path().to_path_buf());

    let error = service.set_photo(&source).unwrap_err();

    assert!(matches!(error, AppError::Validation(_)));
    assert_eq!(std::fs::read_dir(photos.path()).unwrap().count(), 0);
}

#[test]
fn une_image_trop_volumineuse_est_refusee() {
    let dossier = tempfile::tempdir().unwrap();
    let source = dossier.path().join("enorme.png");
    std::fs::write(&source, vec![0_u8; MAX_SOURCE_BYTES + 1]).unwrap();
    let photos = tempfile::tempdir().unwrap();
    let service = ProfileService::new(Memoire::default(), photos.path().to_path_buf());

    let error = service.set_photo(&source).unwrap_err();

    assert!(matches!(error, AppError::Validation(_)));
}

#[test]
fn une_image_trop_grande_est_ramenee_au_cote_maximal_sans_deformation() {
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(1200, 600, image::Rgba([1, 2, 3, 255]))
        .write_to(&mut buffer, image::ImageFormat::Png)
        .unwrap();

    let png = normaliser(&buffer.into_inner()).unwrap();
    let redimensionnee = image::load_from_memory(&png).unwrap();

    // Rapport 2:1 conservé, plus grand côté ramené à la limite.
    assert_eq!(redimensionnee.width(), MAX_SIDE);
    assert_eq!(redimensionnee.height(), MAX_SIDE / 2);
}

#[test]
fn enregistrer_le_profil_ne_touche_pas_a_la_photo() {
    let dossier = tempfile::tempdir().unwrap();
    let source = dossier.path().join("portrait.png");
    std::fs::write(&source, png_de_test()).unwrap();
    let photos = tempfile::tempdir().unwrap();
    let service = ProfileService::new(Memoire::default(), photos.path().to_path_buf());
    let nom = service.set_photo(&source).unwrap().profile.photo.unwrap();

    // Le formulaire de l'écran Profil ne porte pas la photo : il renvoie `None`.
    let payload = service
        .save(&Profile {
            identity: Identity {
                first_name: "Camille".into(),
                name: "Rivet".into(),
                email: "camille@example.fr".into(),
                ..Identity::default()
            },
            ..Profile::default()
        })
        .unwrap();

    assert_eq!(payload.profile.photo.as_deref(), Some(nom.as_str()));
    assert!(photos.path().join(&nom).is_file());
}

#[test]
fn reinitialiser_vide_le_profil_et_sa_photo() {
    let dossier = tempfile::tempdir().unwrap();
    let source = dossier.path().join("portrait.png");
    std::fs::write(&source, png_de_test()).unwrap();
    let photos = tempfile::tempdir().unwrap();
    let service = ProfileService::new(Memoire::default(), photos.path().to_path_buf());
    service
        .save(&Profile {
            identity: Identity {
                first_name: "Camille".into(),
                name: "Rivet".into(),
                email: "camille@example.fr".into(),
                ..Identity::default()
            },
            skills: vec![Skill {
                name: "Rust".into(),
            }],
            ..Profile::default()
        })
        .unwrap();
    let nom = service.set_photo(&source).unwrap().profile.photo.unwrap();

    let payload = service.reset().unwrap();

    assert_eq!(payload.profile, Profile::default());
    assert_eq!(payload.completion, 0);
    assert!(!photos.path().join(&nom).exists());
}

#[test]
fn appliquer_un_import_de_cv_conserve_la_photo() {
    let dossier = tempfile::tempdir().unwrap();
    let source = dossier.path().join("portrait.png");
    std::fs::write(&source, png_de_test()).unwrap();
    let photos = tempfile::tempdir().unwrap();
    let service = ProfileService::new(Memoire::default(), photos.path().to_path_buf());
    let nom = service.set_photo(&source).unwrap().profile.photo.unwrap();

    service.apply_import(&empty_request()).unwrap();

    assert_eq!(
        service.load().unwrap().profile.photo.as_deref(),
        Some(nom.as_str())
    );
    assert!(photos.path().join(&nom).is_file());
}
