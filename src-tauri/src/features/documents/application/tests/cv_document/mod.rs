//! Tests du modèle de CV autonome.

use super::*;
use crate::features::documents::domain::{
    ResumeDocument, ResumeExperienceBlock, ResumeIdentity, ResumeSkillGroup,
};

fn document() -> ResumeDocument {
    ResumeDocument {
        identity: ResumeIdentity {
            full_name: "Alex Exemple".into(),
            title: "Administrateur systèmes".into(),
            email: "alex@example.test".into(),
            ..ResumeIdentity::default()
        },
        profile: "Résumé généré.".into(),
        skill_groups: vec![ResumeSkillGroup {
            id: "skills".into(),
            name: "Techniques".into(),
            items: vec!["Linux".into()],
        }],
        experiences: vec![ResumeExperienceBlock {
            id: "exp-1".into(),
            title: "Développeur".into(),
            company: "Linaïa".into(),
            location: Some("Rennes".into()),
            period: "Juil. 2019 — Oct. 2025".into(),
            bullets: vec!["Une description.".into()],
        }],
        ..ResumeDocument::default()
    }
}

mod test_enrichit_l_experience_du_lieu_et_de_la_periode;
mod test_fusionne_l_identite_et_le_contenu_genere;
