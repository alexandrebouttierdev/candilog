//! Tests du modèle de CV fusionné.

use crate::modules::ia::cv_document::construire;
use crate::modules::ia::cv_model::{
    CvGeneration, GeneratedCv, GeneratedEducation, GeneratedExperience,
};
use crate::shared::profile::{Experience, Language, PersonalInfo, Profile, Project};

fn profil() -> Profile {
    Profile {
        personal: PersonalInfo {
            first_name: "Alex".into(),
            last_name: "Exemple".into(),
            headline: Some("Administrateur systèmes".into()),
            ..PersonalInfo::default()
        },
        experiences: vec![Experience {
            title: "Développeur".into(),
            company: "Linaïa".into(),
            location: Some("Rennes".into()),
            start_date: "2019-07".into(),
            end_date: Some("2025-10".into()),
            ..Experience::default()
        }],
        projects: vec![Project {
            name: "Projet".into(),
            ..Project::default()
        }],
        languages: vec![Language {
            name: "Français".into(),
            level: "natif".into(),
        }],
        ..Profile::default()
    }
}

fn generation() -> CvGeneration {
    CvGeneration {
        cv: GeneratedCv {
            summary: "Résumé généré.".into(),
            skills: vec!["Linux".into()],
            experiences: vec![GeneratedExperience {
                title: "Développeur".into(),
                company: "Linaïa".into(),
                description: "Une description.".into(),
            }],
            education: vec![GeneratedEducation {
                degree: "TSSR".into(),
                school: "ENI".into(),
            }],
        },
        analysis: Default::default(),
    }
}

#[test]
fn fusionne_l_identite_et_le_contenu_genere() {
    let cv = construire(&profil(), &generation());
    assert_eq!(cv.name, "Alex Exemple");
    assert_eq!(cv.subtitle, "Administrateur systèmes");
    assert_eq!(cv.profil, "Résumé généré.");
    assert_eq!(cv.skills, vec!["Linux"]);
}

#[test]
fn enrichit_l_experience_du_lieu_et_de_la_periode() {
    let cv = construire(&profil(), &generation());
    assert_eq!(cv.experiences.len(), 1);
    assert_eq!(cv.experiences[0].meta, "Rennes · Juil. 2019 – Oct. 2025");
    assert_eq!(cv.experiences[0].bullets, vec!["Une description."]);
}
