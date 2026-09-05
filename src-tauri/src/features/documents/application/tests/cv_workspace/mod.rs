//! Tests du document de travail autonome du CV.

use super::*;
use crate::features::ai::domain::{
    AtsAnalysis, AtsContentRecommendation, AtsRecommendation, AtsRecommendationSection,
    ContentRelevance, GeneratedEducation, GeneratedExperience, GeneratedResume, MatchScore,
    ResumeGeneration, StructuredListing,
};
use crate::features::documents::domain::{
    ResumeContentRecommendationAction, ResumeDocument, ResumeIdentity, ResumeProfileItemContent,
    ResumeProposalStatus, ResumeSkillGroup,
};
use crate::features::profile::domain::{
    Certification, Education, Experience, Identity, Language, Profile, Project, Skill,
};

fn profile() -> Profile {
    Profile {
        photo: None,
        identity: Identity {
            first_name: "  Alex ".into(),
            name: " Exemple  ".into(),
            email: "alex@example.test".into(),
            phone: Some("06 00 00 00 00".into()),
            city: Some("Rennes".into()),
            title: Some("Développeur Rust".into()),
            linkedin: Some("https://linkedin.com/in/alex".into()),
            github: Some("https://github.com/alex".into()),
            website: Some("https://alex.example.test".into()),
            ..Identity::default()
        },
        experiences: vec![Experience {
            title: "Développeur".into(),
            company: "Exemple SARL".into(),
            location: Some("Rennes".into()),
            start_date: "2024-01".into(),
            current: true,
            description: Some("Description source".into()),
            ..Experience::default()
        }],
        skills: vec![Skill {
            name: "Rust".into(),
        }],
        education: vec![Education {
            degree: "TSSR".into(),
            school: "ENI".into(),
            location: Some("Rennes".into()),
            start_date: Some("2022-09".into()),
            end_date: Some("2023-06".into()),
            description: Some("Administration systèmes et réseaux".into()),
        }],
        languages: vec![Language {
            name: "Français".into(),
            level: "Natif".into(),
        }],
        projects: vec![Project {
            name: "Candilog".into(),
            description: Some("Application desktop\n· Suivi des candidatures".into()),
            url: Some("https://candilog.fr".into()),
            technologies: Some("Rust · React".into()),
        }],
        certifications: vec![Certification {
            name: "Linux Essentials".into(),
            issuer: Some("LPI".into()),
            date: Some("2024-02".into()),
            url: None,
        }],
    }
}

fn generation() -> ResumeGeneration {
    ResumeGeneration {
        resume: GeneratedResume {
            resume: "Développeur orienté produit et qualité.".into(),
            experiences: vec![GeneratedExperience {
                title: "Développeur".into(),
                company: "Exemple SARL".into(),
                description: "Conception d'une application\n- Tests automatisés".into(),
            }],
            skills: vec!["Rust".into(), "React".into()],
            education: vec![GeneratedEducation {
                degree: "TSSR".into(),
                school: "ENI".into(),
            }],
        },
        analysis: AtsAnalysis {
            content_recommendations: vec![AtsContentRecommendation {
                item_id: "skill-0".into(),
                reason: "Rust est directement demandé par l’offre.".into(),
                relevance: ContentRelevance::VeryRelevant,
            }],
            ..AtsAnalysis::default()
        },
        job_offer: StructuredListing {
            title: "Développeur Rust".into(),
            skills: vec!["Rust".into(), "SQL".into()],
            keywords: vec!["produit".into(), "tests".into()],
            ..StructuredListing::default()
        },
        profile_score: MatchScore {
            total: 99,
            ..MatchScore::default()
        },
        recommendation_error: None,
    }
}

fn minimal_document() -> ResumeDocument {
    ResumeDocument {
        identity: ResumeIdentity {
            full_name: "Alex Exemple".into(),
            title: "Développeur".into(),
            email: "alex@example.test".into(),
            ..ResumeIdentity::default()
        },
        skill_groups: vec![ResumeSkillGroup {
            id: "skills".into(),
            name: "Compétences".into(),
            items: vec!["Rust".into()],
        }],
        ..ResumeDocument::default()
    }
}

/// Compose un poste où `offer_skills` est l'attendu de l'offre et `profile_skills` ce que le
/// profil possède déjà : les compétences absentes des deux ensembles restent manquantes.
fn workspace_avec_offre(offer_skills: Vec<&str>, profile_skills: Vec<&str>) -> ResumeWorkspace {
    let mut source = profile();
    source.skills = profile_skills
        .into_iter()
        .map(|name| Skill { name: name.into() })
        .collect();
    let mut generation = generation();
    generation.resume.skills = source
        .skills
        .iter()
        .map(|skill| skill.name.clone())
        .collect();
    generation.analysis.content_recommendations = source
        .skills
        .iter()
        .enumerate()
        .map(|(index, skill)| AtsContentRecommendation {
            item_id: format!("skill-{index}"),
            reason: format!("{} correspond à l’offre.", skill.name),
            relevance: ContentRelevance::Relevant,
        })
        .collect();
    generation.job_offer.skills = offer_skills.into_iter().map(str::to_owned).collect();
    prepare_workspace(&source, generation, None).unwrap()
}

/// Compose un document dont le profil vaut `original` et porte une unique recommandation IA
/// (« ats-0 ») qui propose de le remplacer par `proposed`. L'offre est alignée sur les
/// compétences générées pour qu'aucune proposition de compétence manquante ne s'intercale.
fn workspace_avec_recommandation(original: &str, proposed: &str) -> ResumeWorkspace {
    let mut generation = generation();
    generation.resume.resume = original.into();
    generation.job_offer.skills = generation.resume.skills.clone();
    generation.analysis.recommendations = vec![AtsRecommendation {
        section: AtsRecommendationSection::Profile,
        item_index: None,
        original_text: original.into(),
        proposed_text: proposed.into(),
    }];
    prepare_workspace(&profile(), generation, None).unwrap()
}

mod applique_une_suggestion_textuelle;
mod calcule_le_gain_d_une_competence;
mod compose_un_document_autonome;
mod refuse_une_suggestion_perimee;
mod reprend_les_competences_du_profil;
mod reprend_les_experiences_et_formations_du_profil;
mod selection_editoriale;
mod valide_les_bornes_du_document;
