//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::profile::{Experience, PersonalInfo, Skill};

fn offer(skills: &[&str], keywords: &[&str], experience: Option<&str>) -> ParsedOffer {
    ParsedOffer {
        title: "Dev".into(),
        skills: skills.iter().map(|s| (*s).to_string()).collect(),
        soft_skills: vec![],
        experience: experience.map(str::to_string),
        keywords: keywords.iter().map(|s| (*s).to_string()).collect(),
    }
}

fn profile_skills(names: &[&str]) -> Profile {
    Profile {
        skills: names
            .iter()
            .map(|n| Skill {
                name: (*n).to_string(),
            })
            .collect(),
        ..Profile::default()
    }
}

fn cv(skills: &[&str], summary: &str) -> crate::modules::ia::cv_model::GeneratedCv {
    crate::modules::ia::cv_model::GeneratedCv {
        summary: summary.into(),
        experiences: vec![],
        skills: skills.iter().map(|s| (*s).to_string()).collect(),
        education: vec![],
    }
}

mod test_score_experience_aucune_exigence_donne_100;
mod test_score_experience_insuffisante_est_proportionnelle;
mod test_score_experience_suffisante_donne_100;
mod test_score_imported_match_partiel_liste_matched_et_missing;
mod test_score_imported_skills_total_donne_100_et_experience_nulle;
mod test_score_imported_total_reweighte_skills_deux_tiers_motscles_un_tiers;
mod test_score_keywords_densite_partielle_donne_50;
mod test_score_offre_sans_skills_ne_divise_pas_par_zero;
mod test_score_skills_match_partiel_donne_50_et_liste_missing;
mod test_score_skills_match_total_donne_100;
