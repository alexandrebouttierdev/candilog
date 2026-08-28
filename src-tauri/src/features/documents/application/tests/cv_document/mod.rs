//! Tests du modèle de CV fusionné.

use super::*;
use crate::features::ia::domain::{CvGenere, ExperienceGeneree, FormationGeneree};
use crate::features::profil::domain::{Experience, Identite, Langue, Profil, Projet};

fn profil() -> Profil {
    Profil {
        identite: Identite {
            prenom: "Alex".into(),
            nom: "Exemple".into(),
            titre: Some("Administrateur systèmes".into()),
            ..Identite::default()
        },
        experiences: vec![Experience {
            intitule: "Développeur".into(),
            entreprise: "Linaïa".into(),
            lieu: Some("Rennes".into()),
            date_debut: "2019-07".into(),
            date_fin: Some("2025-10".into()),
            ..Experience::default()
        }],
        projets: vec![Projet {
            nom: "Projet".into(),
            ..Projet::default()
        }],
        langues: vec![Langue {
            nom: "Français".into(),
            niveau: "natif".into(),
        }],
        ..Profil::default()
    }
}

fn generation() -> CvGenere {
    CvGenere {
        resume: "Résumé généré.".into(),
        competences: vec!["Linux".into()],
        experiences: vec![ExperienceGeneree {
            intitule: "Développeur".into(),
            entreprise: "Linaïa".into(),
            description: "Une description.".into(),
        }],
        formations: vec![FormationGeneree {
            diplome: "TSSR".into(),
            etablissement: "ENI".into(),
        }],
    }
}

mod test_enrichit_l_experience_du_lieu_et_de_la_periode;
mod test_fusionne_l_identite_et_le_contenu_genere;
