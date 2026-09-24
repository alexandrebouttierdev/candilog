//! Adéquation d'une lettre à l'offre (`screens/16`) : la part des exigences de l'offre que
//! la lettre aborde, et ce que le profil permettrait d'y ajouter. Calcul déterministe, sans
//! appel au modèle : une recommandation ne naît que d'un fait vérifié du profil.

use super::normalization::contains_search_term;
use super::scoring::requirement_weight;
use super::{
    search_key, GroundedFact, JobRequirement, ProfileSection, RequirementCategory,
    RequirementImportance, StructuredListing,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ts_rs::TS;

/// Recommandations proposées au plus : au-delà, la colonne devient une liste de tâches.
const MAX_RECOMMENDATIONS: usize = 5;
/// Exigences non couvertes par le profil rappelées au plus.
const MAX_UNSUPPORTED: usize = 5;

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LetterFitRequest {
    pub letter: String,
    /// Offre structurée par `ai_analyze_listing`.
    pub job_offer: StructuredListing,
    /// Arguments exclus de la lettre : ils ne fondent aucune recommandation.
    #[serde(default)]
    pub excluded_sections: Vec<ProfileSection>,
}

/// Ajout proposé : une exigence que la lettre n'aborde pas et qu'un fait du profil prouve.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LetterRecommendation {
    pub id: String,
    pub requirement: String,
    pub importance: RequirementImportance,
    /// Le fait du profil qui permet de l'aborder, tel qu'il y est écrit.
    pub evidence: String,
    /// Consigne envoyée à la correction de lettre, qui reste bornée aux faits vérifiés.
    pub instruction: String,
    /// Points gagnés si la lettre l'aborde.
    pub impact: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LetterFit {
    /// Part pondérée des exigences de l'offre que la lettre aborde, sur 100.
    pub score: u8,
    /// Score atteint si toutes les recommandations étaient suivies.
    pub potential: u8,
    /// Nombre d'exigences pondérées lues dans l'offre ; 0 : rien à mesurer.
    pub requirements: u16,
    pub addressed: Vec<String>,
    pub recommendations: Vec<LetterRecommendation>,
    /// Exigences que ni la lettre ni le profil n'abordent : à ne pas inventer.
    pub unsupported: Vec<String>,
}

/// Exigences pondérées de l'offre, sans doublon. Une offre structurée avant l'introduction
/// des exigences normalisées se rabat sur ses compétences.
fn weighted_requirements(offer: &StructuredListing) -> Vec<(JobRequirement, u16)> {
    let fallback: Vec<JobRequirement>;
    let source = if offer.requirements.is_empty() {
        fallback = offer
            .skills
            .iter()
            .map(|skill| JobRequirement {
                name: skill.clone(),
                category: RequirementCategory::HardSkill,
                ..JobRequirement::default()
            })
            .collect();
        &fallback
    } else {
        &offer.requirements
    };
    let mut seen = HashSet::new();
    source
        .iter()
        .filter(|requirement| !search_key(&requirement.name).is_empty())
        .filter(|requirement| seen.insert(search_key(&requirement.name)))
        .map(|requirement| {
            (
                requirement.clone(),
                requirement_weight(requirement.category, requirement.importance),
            )
        })
        .filter(|(_, weight)| *weight > 0)
        .collect()
}

fn share(part: u32, total: u32) -> u8 {
    if total == 0 {
        return 0;
    }
    u8::try_from(((part * 100 + total / 2) / total).min(100)).unwrap_or(100)
}

#[must_use]
pub fn letter_fit(letter: &str, offer: &StructuredListing, catalog: &[GroundedFact]) -> LetterFit {
    let mut requirements = weighted_requirements(offer);
    // Les plus lourdes d'abord : ce sont elles que l'on recommande en premier.
    requirements.sort_by_key(|(_, weight)| std::cmp::Reverse(*weight));
    let total: u32 = requirements
        .iter()
        .map(|(_, weight)| u32::from(*weight))
        .sum();

    let mut addressed = Vec::new();
    let mut addressed_weight = 0_u32;
    let mut recommendations: Vec<LetterRecommendation> = Vec::new();
    let mut unsupported = Vec::new();
    for (requirement, weight) in &requirements {
        if contains_search_term(letter, &requirement.name) {
            addressed.push(requirement.name.clone());
            addressed_weight += u32::from(*weight);
            continue;
        }
        let evidence = catalog
            .iter()
            .find(|fact| contains_search_term(&fact.text, &requirement.name));
        match evidence {
            Some(fact) if recommendations.len() < MAX_RECOMMENDATIONS => {
                recommendations.push(LetterRecommendation {
                    id: format!("{}:{}", fact.id, search_key(&requirement.name)),
                    requirement: requirement.name.clone(),
                    importance: requirement.importance,
                    evidence: fact.text.clone(),
                    instruction: format!(
                        "Aborde « {} » en t'appuyant uniquement sur ce fait de mon profil : {}",
                        requirement.name, fact.text
                    ),
                    impact: share(u32::from(*weight), total).max(1),
                });
            }
            Some(_) => {}
            None if unsupported.len() < MAX_UNSUPPORTED => {
                unsupported.push(requirement.name.clone());
            }
            None => {}
        }
    }

    let score = share(addressed_weight, total);
    // Somme des gains affichés, pour que « jusqu'à » tombe juste à l'arrondi près.
    let gains: u32 = recommendations
        .iter()
        .map(|recommendation| u32::from(recommendation.impact))
        .sum();
    LetterFit {
        score,
        potential: u8::try_from((u32::from(score) + gains).min(100)).unwrap_or(100),
        requirements: u16::try_from(requirements.len()).unwrap_or(u16::MAX),
        addressed,
        recommendations,
        unsupported,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::ai::domain::GroundedFactKind;

    fn requirement(name: &str, importance: RequirementImportance) -> JobRequirement {
        JobRequirement {
            name: name.into(),
            category: RequirementCategory::HardSkill,
            importance,
            ..JobRequirement::default()
        }
    }

    fn offer() -> StructuredListing {
        StructuredListing {
            requirements: vec![
                requirement("Linux", RequirementImportance::Mandatory),
                requirement("Supervision", RequirementImportance::Important),
                requirement("Kubernetes", RequirementImportance::Important),
            ],
            ..StructuredListing::default()
        }
    }

    fn catalog() -> Vec<GroundedFact> {
        vec![GroundedFact {
            id: "experience:0".into(),
            kind: GroundedFactKind::Experience,
            text: "Technicien chez Ker : supervision de 400 postes".into(),
        }]
    }

    #[test]
    fn mesure_la_part_ponderee_des_exigences_abordees() {
        let fit = letter_fit("Je pratique Linux au quotidien.", &offer(), &catalog());

        // Linux (5) sur Linux (5) + Supervision (3) + Kubernetes (3), à catégorie égale.
        assert_eq!(fit.score, 45);
        assert_eq!(fit.addressed, vec!["Linux".to_owned()]);
        assert_eq!(fit.requirements, 3);
    }

    #[test]
    fn ne_recommande_que_ce_que_le_profil_prouve() {
        let fit = letter_fit("Je pratique Linux au quotidien.", &offer(), &catalog());

        assert_eq!(fit.recommendations.len(), 1);
        let recommendation = &fit.recommendations[0];
        assert_eq!(recommendation.requirement, "Supervision");
        assert!(recommendation.evidence.contains("400 postes"));
        assert!(recommendation
            .instruction
            .contains("uniquement sur ce fait"));
        assert_eq!(fit.potential, fit.score + recommendation.impact);
        // Kubernetes n'est nulle part dans le profil : il est signalé, jamais proposé.
        assert_eq!(fit.unsupported, vec!["Kubernetes".to_owned()]);
    }

    #[test]
    fn une_offre_sans_exigence_ne_donne_pas_de_score() {
        let fit = letter_fit(
            "Madame, Monsieur,",
            &StructuredListing::default(),
            &catalog(),
        );

        assert_eq!(fit.requirements, 0);
        assert_eq!(fit.score, 0);
        assert!(fit.recommendations.is_empty());
    }
}
