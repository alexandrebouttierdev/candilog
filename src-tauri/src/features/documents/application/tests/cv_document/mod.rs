//! Tests du modèle de CV fusionné.

use super::*;
use crate::features::ai::domain::{GeneratedResume, GeneratedExperience, GeneratedEducation};
use crate::features::profile::domain::{Experience, Identity, Language, Profile, Project};

fn profile() -> Profile {
    Profile {
        identity: Identity {
            first_name: "Alex".into(),
            name: "Exemple".into(),
            title: Some("Administrateur systèmes".into()),
            ..Identity::default()
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
            name: "Project".into(),
            ..Project::default()
        }],
        languages: vec![Language {
            name: "Français".into(),
            level: "natif".into(),
        }],
        ..Profile::default()
    }
}

fn generation() -> GeneratedResume {
    GeneratedResume {
        resume: "Résumé généré.".into(),
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
    }
}

mod test_enrichit_l_experience_du_lieu_et_de_la_periode;
mod test_fusionne_l_identite_et_le_contenu_genere;
