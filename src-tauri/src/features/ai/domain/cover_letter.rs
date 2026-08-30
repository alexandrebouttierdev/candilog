//! Lettre de motivation assemblée uniquement depuis un catalogue de faits vérifiés.

use super::normalization::contains_search_term;
use super::{search_key, CoverLetterRequest, ValidateAiOutput, MAX_ITEMS, MAX_ITEM_CHARS};
use crate::core::errors::{AppError, AppResult};
use crate::features::profile::domain::Profile;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroundedFactKind {
    Summary,
    Experience,
    Skill,
    Education,
    Project,
    Certification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GroundedFact {
    pub id: String,
    pub kind: GroundedFactKind,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverLetterPlan {
    #[serde(default, alias = "fact_ids", alias = "selectedFactIds")]
    pub selected_fact_ids: Vec<String>,
    #[serde(default, alias = "motivationKeywords")]
    pub motivation_keywords: Vec<String>,
}

impl ValidateAiOutput for CoverLetterPlan {
    fn validate_ai_output(&self) -> AppResult<()> {
        if self.selected_fact_ids.len() > MAX_ITEMS || self.motivation_keywords.len() > MAX_ITEMS {
            return Err(AppError::Provider(
                "La sélection de faits proposée par l'IA est trop longue.".into(),
            ));
        }
        if self
            .selected_fact_ids
            .iter()
            .chain(&self.motivation_keywords)
            .any(|value| value.chars().count() > MAX_ITEM_CHARS)
        {
            return Err(AppError::Provider(
                "La sélection de faits proposée par l'IA contient un champ trop long.".into(),
            ));
        }
        super::validate_structured_size(self)
    }
}

#[must_use]
pub fn build_fact_catalog(profile: &Profile) -> Vec<GroundedFact> {
    let mut facts = Vec::new();
    if let Some(summary) = profile
        .identity
        .resume
        .as_deref()
        .filter(|summary| !summary.trim().is_empty())
    {
        facts.push(GroundedFact {
            id: "summary:0".into(),
            kind: GroundedFactKind::Summary,
            text: summary.trim().to_owned(),
        });
    }
    facts.extend(
        profile
            .experiences
            .iter()
            .enumerate()
            .map(|(index, experience)| GroundedFact {
                id: format!("experience:{index}"),
                kind: GroundedFactKind::Experience,
                text: match experience
                    .description
                    .as_deref()
                    .filter(|text| !text.trim().is_empty())
                {
                    Some(description) => format!(
                        "{} chez {} : {}",
                        experience.title.trim(),
                        experience.company.trim(),
                        description.trim()
                    ),
                    None => format!(
                        "{} chez {}",
                        experience.title.trim(),
                        experience.company.trim()
                    ),
                },
            }),
    );
    facts.extend(
        profile
            .skills
            .iter()
            .enumerate()
            .filter(|(_, skill)| !skill.name.trim().is_empty())
            .map(|(index, skill)| GroundedFact {
                id: format!("skill:{index}"),
                kind: GroundedFactKind::Skill,
                text: skill.name.trim().to_owned(),
            }),
    );
    facts.extend(
        profile
            .education
            .iter()
            .enumerate()
            .map(|(index, education)| GroundedFact {
                id: format!("education:{index}"),
                kind: GroundedFactKind::Education,
                text: format!("{} à {}", education.degree.trim(), education.school.trim()),
            }),
    );
    facts.extend(profile.projects.iter().enumerate().map(|(index, project)| {
        GroundedFact {
            id: format!("project:{index}"),
            kind: GroundedFactKind::Project,
            text: match project
                .description
                .as_deref()
                .filter(|text| !text.trim().is_empty())
            {
                Some(description) => {
                    format!("{} : {}", project.name.trim(), description.trim())
                }
                None => project.name.trim().to_owned(),
            },
        }
    }));
    facts.extend(
        profile
            .certifications
            .iter()
            .enumerate()
            .map(|(index, certification)| GroundedFact {
                id: format!("certification:{index}"),
                kind: GroundedFactKind::Certification,
                text: match certification
                    .issuer
                    .as_deref()
                    .filter(|text| !text.trim().is_empty())
                {
                    Some(issuer) => {
                        format!(
                            "{} délivrée par {}",
                            certification.name.trim(),
                            issuer.trim()
                        )
                    }
                    None => certification.name.trim().to_owned(),
                },
            }),
    );
    facts.retain(|fact| !fact.text.trim().is_empty());
    facts
}

/// Assemble une lettre depuis des références vérifiées, sans prose factuelle produite par l'IA.
///
/// # Errors
/// Refuse un identifiant de fait inconnu ou une option non prise en charge. Les mots-clés
/// absents du brief sont écartés silencieusement.
pub fn render_grounded_letter(
    catalog: &[GroundedFact],
    plan: &CoverLetterPlan,
    request: &CoverLetterRequest,
) -> AppResult<String> {
    let tone = request.tone.as_deref().unwrap_or("formal");
    if !matches!(tone, "formal" | "casual" | "creative") {
        return Err(AppError::Validation(
            "Le ton de la lettre n'est pas pris en charge.".into(),
        ));
    }
    let fact_limit = match request.length.as_deref().unwrap_or("medium") {
        "short" => 1,
        "medium" => 2,
        "long" => 3,
        _ => {
            return Err(AppError::Validation(
                "La longueur de lettre demandée n'est pas prise en charge.".into(),
            ));
        }
    };
    let by_id: HashMap<&str, &GroundedFact> = catalog
        .iter()
        .map(|fact| (fact.id.as_str(), fact))
        .collect();
    let mut selected = Vec::new();
    let mut seen_ids = HashSet::new();
    for id in &plan.selected_fact_ids {
        let fact = by_id.get(id.as_str()).ok_or_else(|| {
            AppError::Provider("La réponse IA référence un fait inconnu du profil.".into())
        })?;
        if seen_ids.insert(id.as_str()) && selected.len() < fact_limit {
            selected.push(*fact);
        }
    }

    let brief = [
        request.company.as_deref().unwrap_or_default(),
        request.job_title.as_deref().unwrap_or_default(),
        request.context.as_deref().unwrap_or_default(),
    ]
    .join(" ");
    let mut keywords = Vec::new();
    let mut seen_keywords = HashSet::new();
    for keyword in &plan.motivation_keywords {
        let key = search_key(keyword);
        // Un mot-clé reformulé par le modèle (« innovation » pour « innovant ») est écarté,
        // pas fatal : la lettre ne cite que le brief, et une paraphrase ne doit pas faire
        // échouer toute la rédaction.
        if key.is_empty() || !contains_search_term(&brief, keyword) {
            continue;
        }
        if seen_keywords.insert(key) && keywords.len() < 5 {
            keywords.push(keyword.trim());
        }
    }

    let company = request
        .company
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("votre entreprise");
    let job = request
        .job_title
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("ce poste");
    let mut paragraphs = vec![format!(
        "Madame, Monsieur,\n\nJe vous adresse ma candidature au poste de {job} au sein de {company}."
    )];
    paragraphs.extend(selected.into_iter().map(|fact| fact_sentence(fact, tone)));
    if !keywords.is_empty() {
        paragraphs.push(format!(
            "Votre besoin autour de {} motive particulièrement ma candidature.",
            join_french(&keywords)
        ));
    }
    paragraphs.push(match tone {
        "casual" => "Je serais ravi d'échanger avec vous afin de vous présenter ma démarche et mes motivations.\n\nCordialement,".into(),
        "creative" => "Je serais heureux de transformer cette candidature en échange concret avec votre équipe.\n\nCordialement,".into(),
        _ => "Je serais heureux de pouvoir échanger avec vous afin de détailler ma candidature.\n\nVeuillez agréer, Madame, Monsieur, mes salutations distinguées.".into(),
    });
    Ok(paragraphs.join("\n\n"))
}

fn fact_sentence(fact: &GroundedFact, tone: &str) -> String {
    match (fact.kind, tone) {
        (GroundedFactKind::Summary, _) => format!("Mon projet professionnel : {}.", fact.text),
        (GroundedFactKind::Experience, "casual" | "creative") => {
            format!("Mon parcours comprend notamment {}.", fact.text)
        }
        (GroundedFactKind::Experience, _) => {
            format!("Mon expérience comprend notamment {}.", fact.text)
        }
        (GroundedFactKind::Skill, _) => format!("Je peux notamment mobiliser {}.", fact.text),
        (GroundedFactKind::Education, _) => format!("Ma formation inclut {}.", fact.text),
        (GroundedFactKind::Project, _) => format!("J'ai également mené le projet {}.", fact.text),
        (GroundedFactKind::Certification, _) => {
            format!(
                "Mon parcours comprend aussi la certification {}.",
                fact.text
            )
        }
    }
}

fn join_french(values: &[&str]) -> String {
    match values {
        [] => String::new(),
        [only] => (*only).to_owned(),
        [first, second] => format!("{first} et {second}"),
        _ => format!(
            "{} et {}",
            values[..values.len() - 1].join(", "),
            values[values.len() - 1]
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::errors::AppError;
    use crate::features::ai::domain::CoverLetterRequest;

    fn catalog() -> Vec<GroundedFact> {
        vec![GroundedFact {
            id: "experience:0".into(),
            kind: GroundedFactKind::Experience,
            text: "Ingénieure chez Nova — APIs Rust".into(),
        }]
    }

    fn request() -> CoverLetterRequest {
        CoverLetterRequest {
            generation_id: "test".into(),
            company: Some("Acme".into()),
            job_title: Some("Développeuse Rust".into()),
            tone: Some("formal".into()),
            length: Some("medium".into()),
            context: Some("Acme recherche une développeuse Rust pour ses APIs".into()),
            previous_cover_letter: None,
            instruction: None,
        }
    }

    #[test]
    fn une_reference_de_fait_inconnue_est_refusee() {
        let plan = CoverLetterPlan {
            selected_fact_ids: vec!["experience:inconnue".into()],
            motivation_keywords: vec![],
        };

        assert!(matches!(
            render_grounded_letter(&catalog(), &plan, &request()),
            Err(AppError::Provider(_))
        ));
    }

    #[test]
    fn la_lettre_rendue_ne_contient_que_les_faits_du_catalogue() {
        let plan = CoverLetterPlan {
            selected_fact_ids: vec!["experience:0".into()],
            motivation_keywords: vec!["APIs".into()],
        };

        let text = render_grounded_letter(&catalog(), &plan, &request()).unwrap();

        assert!(text.contains("Nova"));
        assert!(text.contains("APIs"));
        assert!(!text.contains("Google"));
    }

    #[test]
    fn un_mot_cle_absent_du_brief_est_ecarte_sans_faire_echouer_la_lettre() {
        let plan = CoverLetterPlan {
            selected_fact_ids: vec!["experience:0".into()],
            motivation_keywords: vec!["Kubernetes".into(), "APIs".into()],
        };

        let text = render_grounded_letter(&catalog(), &plan, &request()).unwrap();

        assert!(!text.contains("Kubernetes"));
        assert!(text.contains("Votre besoin autour de APIs"));
    }

    #[test]
    fn un_fragment_de_mot_n_est_pas_un_mot_cle_du_brief() {
        let mut request = request();
        request.context = Some("Un projet ongoing".into());
        let plan = CoverLetterPlan {
            selected_fact_ids: vec![],
            motivation_keywords: vec!["go".into()],
        };

        let text = render_grounded_letter(&catalog(), &plan, &request).unwrap();

        assert!(!text.contains("Votre besoin autour de"));
    }
}
