use super::*;
use crate::features::profil::domain::{
    Certification, Competence, Experience, Formation, Langue, Projet,
};
use std::sync::Mutex;

#[derive(Default)]
struct Memoire {
    profil: Mutex<Option<Profil>>,
}

impl ProfilRepository for Memoire {
    fn obtenir(&self) -> AppResult<(Profil, Option<String>)> {
        Ok((
            self.profil.lock().unwrap().clone().unwrap_or_default(),
            None,
        ))
    }

    fn enregistrer(&self, profil: &Profil) -> AppResult<(Profil, String)> {
        *self.profil.lock().unwrap() = Some(profil.clone());
        Ok((profil.clone(), "2026-08-28T10:00:00Z".into()))
    }
}

#[test]
fn profil_vide_a_un_score_nul_et_sept_pistes() {
    let service = ProfilService::new(Memoire::default());

    let charge = service.charger().unwrap();

    assert_eq!(charge.completion, 0);
    assert_eq!(charge.sections_incompletes.len(), 7);
}

#[test]
fn profil_complet_atteint_cent() {
    let profil = Profil {
        identite: Identite {
            prenom: "Camille".into(),
            nom: "Rivet".into(),
            email: "camille@example.fr".into(),
            ..Identite::default()
        },
        experiences: vec![Experience {
            intitule: "Développeuse".into(),
            entreprise: "Nova".into(),
            date_debut: "2024-01".into(),
            poste_actuel: true,
            ..Experience::default()
        }],
        competences: vec![Competence { nom: "Rust".into() }],
        formations: vec![Formation {
            diplome: "Master".into(),
            etablissement: "Université".into(),
            ..Formation::default()
        }],
        langues: vec![Langue {
            nom: "Français".into(),
            niveau: "Natif".into(),
        }],
        projets: vec![Projet {
            nom: "Candilog".into(),
            ..Projet::default()
        }],
        certifications: vec![Certification {
            nom: "AWS".into(),
            ..Certification::default()
        }],
    };
    let service = ProfilService::new(Memoire::default());

    let charge = service.enregistrer(&profil).unwrap();

    assert_eq!(charge.completion, 100);
    assert!(charge.sections_incompletes.is_empty());
}

#[test]
fn une_experience_sans_debut_est_refusee() {
    let profil = Profil {
        experiences: vec![Experience {
            intitule: "Développeuse".into(),
            entreprise: "Nova".into(),
            ..Experience::default()
        }],
        ..Profil::default()
    };

    let erreur = ProfilService::new(Memoire::default())
        .enregistrer(&profil)
        .unwrap_err();

    assert!(matches!(erreur, AppError::Validation(_)));
}

#[test]
fn une_identite_avec_email_invalide_est_refusee() {
    let profil = Profil {
        identite: Identite {
            email: "camille@localhost".into(),
            ..Identite::default()
        },
        ..Profil::default()
    };

    let erreur = ProfilService::new(Memoire::default())
        .enregistrer(&profil)
        .unwrap_err();

    assert!(matches!(erreur, AppError::Validation(_)));
}

#[test]
fn une_entree_legacy_incomplete_ne_gonfle_pas_le_score() {
    let repo = Memoire {
        profil: Mutex::new(Some(Profil {
            experiences: vec![Experience {
                intitule: "Développeuse".into(),
                ..Experience::default()
            }],
            competences: vec![Competence::default()],
            ..Profil::default()
        })),
    };

    let charge = ProfilService::new(repo).charger().unwrap();

    assert_eq!(charge.completion, 0);
    assert!(charge
        .sections_incompletes
        .contains(&"une expérience".into()));
    assert!(charge
        .sections_incompletes
        .contains(&"vos compétences".into()));
}
