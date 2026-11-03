//! Score ATS local et déterministe, sans appel réseau.

use super::normalization::{contains_search_term, deduplicate_labels};
use super::{
    search_key, AtsAnalysis, AtsBreakdownItem, GeneratedResume, JobRequirement, MatchScore,
    RequirementCategory, RequirementEvaluation, RequirementImportance, RequirementMatchKind,
    StructuredListing,
};
use crate::features::profile::domain::Profile;
use chrono::Datelike;
use serde::Serialize;
use std::collections::HashSet;

const EQUIVALENT_CREDIT: u8 = 90;
const TRANSFERABLE_CREDIT: u8 = 45;
const PARTIAL_CREDIT: u8 = 55;

/// Familles multi-domaines d'équivalence / transfert.
/// Données structurées génériques — pas de branche `if skill == "react"`.
const LEGACY_TRANSFER_FAMILIES: &[&[&str]] = &[
    &["react", "angular", "vue", "svelte", "ember"],
    &[
        "ci/cd",
        "cicd",
        "gitlab ci",
        "github actions",
        "integration continue",
        "intégration continue",
        "pipelines",
    ],
    &["java", "j2ee", "spring", "spring boot", "jakarta ee"],
    &[
        "excel",
        "calc",
        "google sheets",
        "sheets",
        "numbers",
        "libreoffice calc",
    ],
    &[
        "word",
        "writer",
        "google docs",
        "pages",
        "libreoffice writer",
    ],
    &[
        "salesforce",
        "hubspot",
        "dynamics 365",
        "pipedrive",
        "zoho crm",
    ],
    &["sap", "odoo", "microsoft dynamics", "oracle ebs"],
    &["autocad", "solidworks", "catia", "revit", "archicad"],
    &["photoshop", "gimp", "affinity photo", "lightroom"],
    &["figma", "sketch", "adobe xd", "penpot"],
    &["quickbooks", "sage", "xero", "ciel"],
    &["power bi", "tableau", "looker", "qlik"],
];

/// Libellés typiques de savoir-être / méthodes à sortir des compétences dures.
/// Évite qu'Agile, qualité, MOA, UX… diluent le % compétences (score ~31).
const SOFT_REQUIREMENT_MARKERS: &[&str] = &[
    "agile",
    "scrum",
    "kanban",
    "code review",
    "qualite",
    "qualite logicielle",
    "autonomie",
    "esprit d equipe",
    "communication",
    "rigueur",
    "moa",
    "moe",
    "ux",
    "ui ux",
    "lignes directrices",
    "manuel",
    "manuels",
    "exploitation",
    "sensibilite",
    "methodes et outils ia",
    "outils ia",
    "intelligence artificielle",
    "industrialisation",
    "cadrage",
    "reunion",
    "reunions",
];

/// Mots trop génériques pour servir de token de proximité métier.
const TITLE_STOPWORDS: &[&str] = &[
    "de",
    "du",
    "des",
    "la",
    "le",
    "les",
    "un",
    "une",
    "et",
    "en",
    "au",
    "aux",
    "a",
    "the",
    "and",
    "or",
    "of",
    "in",
    "for",
    "to",
    "junior",
    "senior",
    "confirme",
    "confirmee",
    "experimente",
    "experimentee",
    "h",
    "f",
    "hf",
    "fh",
];

mod catalog;
mod grounding;

pub use catalog::{
    ground_ats_recommendations, ground_content_recommendations, profile_content_catalog,
    ProfileContentCatalogEntry,
};
use grounding::*;
pub use grounding::{ground_extracted_listing, ground_generated_resume, ground_imported_resume};

#[must_use]
pub fn profile_score(profile: &Profile, job_offer: &StructuredListing) -> MatchScore {
    let names: Vec<&str> = profile
        .skills
        .iter()
        .map(|skill| skill.name.as_str())
        .chain(profile.certifications.iter().map(|item| item.name.as_str()))
        .chain(profile.languages.iter().map(|item| item.name.as_str()))
        .chain(profile.education.iter().map(|item| item.degree.as_str()))
        .filter(|name| !search_key(name).is_empty())
        .collect();
    let evidence = candidate_evidence(profile);
    score_against_offer(
        &names,
        &evidence,
        profile_title(profile),
        Some(annees_experience(profile)),
        profile
            .identity
            .city
            .as_deref()
            .or(profile.identity.address.as_deref()),
        true,
        job_offer,
    )
}

/// Une compétence de l'offre est couverte dès qu'une compétence du candidat la contient
/// comme mot entier.
///
/// L'égalité stricte des clés normalisées demandait au candidat d'avoir écrit exactement le
/// libellé de l'offre : « VMware vSphere » ne couvrait pas « VMware », « Windows Server
/// 2019 » ne couvrait pas « Windows ». Le score des profils réels s'effondrait, et l'éditeur
/// proposait d'« ajouter » une compétence déjà présente sous son nom complet. La frontière
/// de mot est celle de [`contains_search_term`] : « Java » ne couvre toujours pas
/// « JavaScript ».
fn skill_couverte(candidat: &[&str], attendue: &str) -> bool {
    candidat
        .iter()
        .any(|nom| contains_search_term(nom, attendue))
}

#[must_use]
pub fn score_resume_imported(
    resume: &GeneratedResume,
    job_offer: &StructuredListing,
) -> MatchScore {
    score_resume_imported_with_source(resume, job_offer, None)
}

/// Comme [`score_resume_imported`], en enrichissant l'évidence avec le texte brut du PDF.
///
/// Sans ça, un JSON de structuration incomplet (compétences / projets omis) fait chuter
/// le score vers ~5/100 alors que CI/CD, Agile ou Angular figurent bien dans le PDF.
#[must_use]
pub fn score_resume_imported_with_source(
    resume: &GeneratedResume,
    job_offer: &StructuredListing,
    source: Option<&str>,
) -> MatchScore {
    let names: Vec<&str> = resume
        .skills
        .iter()
        .map(String::as_str)
        .chain(resume.education.iter().map(|item| item.degree.as_str()))
        .filter(|name| !search_key(name).is_empty())
        .collect();
    let evidence = match source {
        Some(src) if !src.trim().is_empty() => {
            search_key(&format!("{} {}", resume_text_raw(resume), src))
        }
        _ => resume_text(resume),
    };
    score_against_offer(
        &names,
        &evidence,
        &infer_title_from_resume(resume),
        infer_annees_from_parts(resume, source),
        None,
        source.is_some_and(|value| !value.trim().is_empty()),
        job_offer,
    )
}

/// Score déterministe partagé (profil complet ou CV importé).
fn score_against_offer(
    skill_names: &[&str],
    evidence: &str,
    title: &str,
    annees: Option<usize>,
    location: Option<&str>,
    allow_unstructured_skill_evidence: bool,
    job_offer: &StructuredListing,
) -> MatchScore {
    let (requirements, legacy) = normalized_requirements(job_offer);
    let candidate = CandidateContext {
        skill_names,
        evidence,
        title,
        years: annees,
        location,
        allow_unstructured_skill_evidence,
    };
    let evaluations: Vec<RequirementEvaluation> = requirements
        .iter()
        .map(|requirement| evaluate_requirement(requirement, &candidate, legacy))
        .collect();

    let (weighted_sum, total_weight) = evaluations.iter().fold(
        (0_u32, 0_u32),
        |(sum, total_weight), evaluation| match evaluation.score {
            Some(score) => {
                let weight = requirement_weight(evaluation.category, evaluation.importance);
                (
                    sum + u32::from(score) * u32::from(weight),
                    total_weight + u32::from(weight),
                )
            }
            None => (sum, total_weight),
        },
    );
    let raw_total = weighted_sum
        .saturating_add(total_weight / 2)
        .checked_div(total_weight)
        .unwrap_or_default()
        .min(100) as u8;
    let missing_critical = evaluations.iter().any(|evaluation| {
        evaluation.category.is_regulated()
            && evaluation.importance == RequirementImportance::Mandatory
            && evaluation.match_kind == RequirementMatchKind::Missing
    });
    let total = if missing_critical {
        raw_total.min(35)
    } else {
        raw_total
    };

    let skills = category_score(
        &evaluations,
        &[
            RequirementCategory::HardSkill,
            RequirementCategory::Tool,
            RequirementCategory::Methodology,
        ],
    );
    let experience = category_score(&evaluations, &[RequirementCategory::Experience]);
    let ats = category_score(
        &evaluations,
        &[
            RequirementCategory::Responsibility,
            RequirementCategory::SoftSkill,
            RequirementCategory::Industry,
        ],
    );
    let mut present = Vec::new();
    let mut missing = Vec::new();
    for evaluation in &evaluations {
        if !matches!(
            evaluation.category,
            RequirementCategory::HardSkill
                | RequirementCategory::Tool
                | RequirementCategory::Methodology
                | RequirementCategory::Education
                | RequirementCategory::Certification
                | RequirementCategory::License
                | RequirementCategory::Language
        ) {
            continue;
        }
        match evaluation.match_kind {
            RequirementMatchKind::Exact
            | RequirementMatchKind::Equivalent
            | RequirementMatchKind::Transferable
            | RequirementMatchKind::Partial => present.push(evaluation.requirement.clone()),
            RequirementMatchKind::Missing => missing.push(evaluation.requirement.clone()),
            RequirementMatchKind::Unknown => {}
        }
    }

    MatchScore {
        total,
        skills,
        experience,
        ats,
        present: deduplicate_labels(&present),
        missing: deduplicate_labels(&missing),
        breakdown: build_breakdown(&evaluations),
        evaluations,
        critical_requirements_penalty: raw_total.saturating_sub(total),
    }
}

fn normalized_requirements(job_offer: &StructuredListing) -> (Vec<JobRequirement>, bool) {
    let legacy = job_offer.requirements.is_empty();
    let mut requirements = if legacy {
        let (hard, soft, keywords) = partition_offer_requirements(job_offer);
        let mut values = hard
            .into_iter()
            .map(|name| JobRequirement {
                name,
                category: RequirementCategory::HardSkill,
                importance: RequirementImportance::Important,
                ..JobRequirement::default()
            })
            .chain(soft.into_iter().map(|name| JobRequirement {
                name,
                category: RequirementCategory::SoftSkill,
                importance: RequirementImportance::Preferred,
                ..JobRequirement::default()
            }))
            .chain(keywords.into_iter().map(|name| JobRequirement {
                name,
                category: RequirementCategory::Responsibility,
                importance: RequirementImportance::Preferred,
                ..JobRequirement::default()
            }))
            .collect::<Vec<_>>();
        if let Some(experience) = job_offer.experience.as_deref() {
            let minimum_years = first_entier(experience);
            if minimum_years > 0 {
                values.push(JobRequirement {
                    name: experience.trim().to_owned(),
                    category: RequirementCategory::Experience,
                    importance: RequirementImportance::Important,
                    minimum_years: u8::try_from(minimum_years).ok(),
                    ..JobRequirement::default()
                });
            }
        }
        values
    } else {
        job_offer.requirements.clone()
    };

    if meaningful_occupation(&job_offer.title)
        && !requirements
            .iter()
            .any(|item| item.category == RequirementCategory::Occupation)
    {
        requirements.push(JobRequirement {
            name: job_offer.title.trim().to_owned(),
            category: RequirementCategory::Occupation,
            importance: RequirementImportance::Important,
            ..JobRequirement::default()
        });
    }
    if let Some(location) = job_offer
        .location
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        if !requirements
            .iter()
            .any(|item| item.category == RequirementCategory::Location)
        {
            requirements.push(JobRequirement {
                name: location.trim().to_owned(),
                category: RequirementCategory::Location,
                importance: RequirementImportance::Preferred,
                ..JobRequirement::default()
            });
        }
    }

    let mut deduplicated = Vec::<JobRequirement>::new();
    for mut requirement in requirements {
        requirement.name = requirement.name.trim().to_owned();
        if requirement.name.is_empty()
            || matches!(
                requirement.importance,
                RequirementImportance::Contextual | RequirementImportance::Informational
            )
        {
            continue;
        }
        if requirement.mandatory {
            requirement.importance = RequirementImportance::Mandatory;
        }
        requirement.transferable_from = deduplicate_labels(&requirement.transferable_from);
        if let Some(existing) = deduplicated
            .iter_mut()
            .find(|existing| labels_overlap(&existing.name, &requirement.name))
        {
            existing.importance = existing.importance.max(requirement.importance);
            existing.mandatory |= requirement.mandatory;
            existing.minimum_years = existing.minimum_years.max(requirement.minimum_years);
            existing
                .transferable_from
                .extend(requirement.transferable_from);
            existing.transferable_from = deduplicate_labels(&existing.transferable_from);
        } else {
            deduplicated.push(requirement);
        }
    }
    (deduplicated, legacy)
}

fn meaningful_occupation(title: &str) -> bool {
    let key = search_key(title);
    !key.is_empty() && !matches!(key.as_str(), "poste" | "emploi" | "offre")
}

fn labels_overlap(left: &str, right: &str) -> bool {
    let left_key = search_key(left);
    let right_key = search_key(right);
    !left_key.is_empty()
        && (left_key == right_key
            || contains_search_term(left, right)
            || contains_search_term(right, left))
}

struct CandidateContext<'a> {
    skill_names: &'a [&'a str],
    evidence: &'a str,
    title: &'a str,
    years: Option<usize>,
    location: Option<&'a str>,
    allow_unstructured_skill_evidence: bool,
}

fn evaluate_requirement(
    requirement: &JobRequirement,
    candidate: &CandidateContext<'_>,
    legacy: bool,
) -> RequirementEvaluation {
    let importance = if requirement.mandatory {
        RequirementImportance::Mandatory
    } else {
        requirement.importance
    };
    let result = match requirement.category {
        RequirementCategory::Experience => match (
            requirement.minimum_years.map(usize::from).or_else(|| {
                let parsed = first_entier(&requirement.name);
                (parsed > 0).then_some(parsed)
            }),
            candidate.years,
        ) {
            (Some(required), Some(actual)) => {
                let score = actual
                    .saturating_mul(100)
                    .checked_div(required)
                    .map_or(0, |value| value.min(100) as u8);
                let kind = if score == 100 {
                    RequirementMatchKind::Exact
                } else if score > 0 {
                    RequirementMatchKind::Partial
                } else {
                    RequirementMatchKind::Missing
                };
                (kind, Some(score), Some(format!("{actual} ans")))
            }
            _ => (RequirementMatchKind::Unknown, None, None),
        },
        RequirementCategory::Occupation => {
            match occupation_proximity(candidate.title, &requirement.name) {
                Some(score) if score == 100 => (
                    RequirementMatchKind::Exact,
                    Some(score),
                    Some(candidate.title.to_owned()),
                ),
                Some(score) if score >= PARTIAL_CREDIT => (
                    RequirementMatchKind::Equivalent,
                    Some(score.max(EQUIVALENT_CREDIT)),
                    Some(candidate.title.to_owned()),
                ),
                Some(score) if score > 0 => (
                    RequirementMatchKind::Partial,
                    Some(score.max(PARTIAL_CREDIT)),
                    Some(candidate.title.to_owned()),
                ),
                Some(_) => (RequirementMatchKind::Missing, Some(0), None),
                None => (RequirementMatchKind::Unknown, None, None),
            }
        }
        RequirementCategory::Location => {
            let candidate_location = candidate.location.unwrap_or(candidate.evidence);
            if contains_term(candidate_location, &requirement.name) {
                (
                    RequirementMatchKind::Exact,
                    Some(100),
                    Some(requirement.name.clone()),
                )
            } else {
                (RequirementMatchKind::Missing, Some(0), None)
            }
        }
        _ => evaluate_text_requirement(
            requirement,
            candidate.skill_names,
            candidate.evidence,
            candidate.allow_unstructured_skill_evidence,
            legacy,
        ),
    };
    RequirementEvaluation {
        requirement: requirement.name.clone(),
        category: requirement.category,
        importance,
        match_kind: result.0,
        score: result.1,
        evidence: result.2,
    }
}

fn evaluate_text_requirement(
    requirement: &JobRequirement,
    skill_names: &[&str],
    evidence: &str,
    allow_unstructured_skill_evidence: bool,
    legacy: bool,
) -> (RequirementMatchKind, Option<u8>, Option<String>) {
    if let Some(exact) = skill_names
        .iter()
        .find(|candidate| search_key(candidate) == search_key(&requirement.name))
    {
        return (
            RequirementMatchKind::Exact,
            Some(100),
            Some((*exact).to_string()),
        );
    }
    if let Some(equivalent) = skill_names
        .iter()
        .find(|candidate| labels_overlap(candidate, &requirement.name))
    {
        return (
            RequirementMatchKind::Equivalent,
            Some(100),
            Some((*equivalent).to_string()),
        );
    }
    let strict_listing = matches!(
        requirement.category,
        RequirementCategory::HardSkill
            | RequirementCategory::Tool
            | RequirementCategory::Methodology
    );
    if (!strict_listing || allow_unstructured_skill_evidence)
        && contains_term(evidence, &requirement.name)
    {
        return (
            RequirementMatchKind::Exact,
            Some(100),
            Some(requirement.name.clone()),
        );
    }

    if !requirement.category.is_regulated() {
        if let Some(term) = requirement
            .transferable_from
            .iter()
            .find(|term| {
                skill_couverte(skill_names, term)
                    || (allow_unstructured_skill_evidence && contains_term(evidence, term))
            })
            .cloned()
            .or_else(|| {
                let legacy_evidence = if allow_unstructured_skill_evidence {
                    evidence
                } else {
                    ""
                };
                legacy.then(|| {
                    legacy_transferable_requirement(skill_names, legacy_evidence, &requirement.name)
                })?
            })
        {
            return (
                RequirementMatchKind::Transferable,
                Some(TRANSFERABLE_CREDIT),
                Some(term),
            );
        }
    }

    if matches!(
        requirement.category,
        RequirementCategory::Responsibility
            | RequirementCategory::SoftSkill
            | RequirementCategory::Industry
            | RequirementCategory::Availability
            | RequirementCategory::Other
    ) && token_coverage(evidence, &requirement.name) >= 50
    {
        return (RequirementMatchKind::Partial, Some(PARTIAL_CREDIT), None);
    }
    (RequirementMatchKind::Missing, Some(0), None)
}

fn token_coverage(evidence: &str, requirement: &str) -> u8 {
    let normalized = search_key(requirement);
    let tokens: Vec<&str> = normalized
        .split_whitespace()
        .filter(|token| token.len() >= 4)
        .filter(|token| !TITLE_STOPWORDS.contains(token))
        .collect();
    percentage(
        tokens
            .iter()
            .filter(|token| contains_search_term(evidence, token))
            .count(),
        tokens.len(),
    )
    .unwrap_or(0)
}

pub(super) const fn requirement_weight(
    category: RequirementCategory,
    importance: RequirementImportance,
) -> u16 {
    let category_weight = match category {
        RequirementCategory::Occupation => 20,
        RequirementCategory::Experience => 16,
        RequirementCategory::Education
        | RequirementCategory::Certification
        | RequirementCategory::License => 18,
        RequirementCategory::HardSkill => 14,
        RequirementCategory::Responsibility => 12,
        RequirementCategory::Tool | RequirementCategory::Methodology => 11,
        RequirementCategory::Language => 10,
        RequirementCategory::SoftSkill => 7,
        RequirementCategory::Industry => 5,
        RequirementCategory::Location | RequirementCategory::Availability => 4,
        RequirementCategory::Other => 6,
    };
    let importance_weight = match importance {
        RequirementImportance::Mandatory => 5,
        RequirementImportance::Important => 3,
        RequirementImportance::Preferred => 2,
        RequirementImportance::Optional => 1,
        RequirementImportance::Contextual | RequirementImportance::Informational => 0,
    };
    category_weight * importance_weight
}

fn category_score(
    evaluations: &[RequirementEvaluation],
    categories: &[RequirementCategory],
) -> Option<u8> {
    let (sum, weight) = evaluations
        .iter()
        .filter(|item| categories.contains(&item.category))
        .filter_map(|item| {
            item.score.map(|score| {
                let weight = requirement_weight(item.category, item.importance);
                (u32::from(score) * u32::from(weight), u32::from(weight))
            })
        })
        .fold((0_u32, 0_u32), |(sum, weight), item| {
            (sum + item.0, weight + item.1)
        });
    (weight > 0).then(|| ((sum + weight / 2) / weight).min(100) as u8)
}

fn build_breakdown(evaluations: &[RequirementEvaluation]) -> Vec<AtsBreakdownItem> {
    const CATEGORIES: &[(RequirementCategory, &str)] = &[
        (RequirementCategory::Occupation, "Métier / fonction"),
        (RequirementCategory::Experience, "Expérience"),
        (RequirementCategory::HardSkill, "Savoir-faire"),
        (RequirementCategory::Responsibility, "Missions"),
        (RequirementCategory::Tool, "Outils"),
        (RequirementCategory::Methodology, "Méthodes"),
        (RequirementCategory::SoftSkill, "Compétences relationnelles"),
        (RequirementCategory::Education, "Diplômes"),
        (RequirementCategory::Certification, "Certifications"),
        (RequirementCategory::License, "Permis et habilitations"),
        (RequirementCategory::Language, "Langues"),
        (RequirementCategory::Industry, "Secteur"),
        (RequirementCategory::Location, "Localisation"),
        (RequirementCategory::Availability, "Disponibilité"),
        (RequirementCategory::Other, "Autres critères"),
    ];
    CATEGORIES
        .iter()
        .filter_map(|(category, label)| {
            let category_evaluations: Vec<&RequirementEvaluation> = evaluations
                .iter()
                .filter(|item| item.category == *category && item.score.is_some())
                .collect();
            if category_evaluations.is_empty() {
                return None;
            }
            let weight = category_evaluations
                .iter()
                .map(|item| requirement_weight(item.category, item.importance))
                .sum();
            category_score(evaluations, &[*category]).map(|score| AtsBreakdownItem {
                category: *category,
                label: (*label).to_owned(),
                score,
                weight,
            })
        })
        .collect()
}

fn legacy_transferable_requirement(
    skill_names: &[&str],
    evidence: &str,
    required: &str,
) -> Option<String> {
    let required_key = search_key(required);
    if required_key.is_empty() {
        return None;
    }
    for family in LEGACY_TRANSFER_FAMILIES {
        let in_family = family.iter().any(|member| {
            search_key(member) == required_key
                || contains_search_term(required, member)
                || contains_search_term(member, required)
        });
        if !in_family {
            continue;
        }
        for member in *family {
            let member_key = search_key(member);
            if member_key.is_empty() || member_key == required_key {
                continue;
            }
            if skill_couverte(skill_names, member) || contains_term(evidence, member) {
                return Some((*member).to_owned());
            }
        }
    }
    None
}

fn occupation_proximity(candidate_title: &str, offer_title: &str) -> Option<u8> {
    let left = significant_tokens(candidate_title);
    let right = significant_tokens(offer_title);
    if left.is_empty() || right.is_empty() {
        return None;
    }
    let overlap = tokens_overlap(&left, &right);
    let denom = left.len().max(right.len());
    Some(((overlap * 100) / denom).min(100) as u8)
}

fn significant_tokens(value: &str) -> Vec<String> {
    search_key(value)
        .split_whitespace()
        .filter(|token| token.len() >= 3)
        .filter(|token| !TITLE_STOPWORDS.contains(token))
        .map(|token| stem_titre(token.to_owned()))
        .collect()
}

/// Rapproche les variantes de genre FR courantes (développeur / développeuse).
fn stem_titre(token: String) -> String {
    for suffix in ["euse", "eure", "rice", "iere", "ier"] {
        if let Some(stem) = token.strip_suffix(suffix) {
            if stem.len() >= 4 {
                return stem.to_owned();
            }
        }
    }
    // développeur → developp (strip eur)
    for suffix in ["eur", "aux", "ais", "ait"] {
        if let Some(stem) = token.strip_suffix(suffix) {
            if stem.len() >= 4 {
                return stem.to_owned();
            }
        }
    }
    token
}

fn tokens_overlap(left: &[String], right: &[String]) -> usize {
    left.iter()
        .filter(|token| {
            right.iter().any(|other| {
                *token == other
                    || (token.len() >= 5
                        && other.len() >= 5
                        && (other.starts_with(token.as_str()) || token.starts_with(other.as_str())))
            })
        })
        .count()
}

/// Sépare compétences dures, savoir-être et mots-clés pour éviter la dilution du score.
fn partition_offer_requirements(
    job_offer: &StructuredListing,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut hard = Vec::new();
    let mut soft = deduplicate_labels(&job_offer.soft_skills);
    let mut keywords = deduplicate_labels(&job_offer.keywords);

    for skill in deduplicate_labels(&job_offer.skills) {
        if looks_like_soft_requirement(&skill) {
            soft.push(skill);
        } else {
            hard.push(skill);
        }
    }
    soft = deduplicate_labels(&soft);

    // Mots-clés trop « soft » ou déjà en hard → hors double compte.
    keywords.retain(|keyword| {
        !looks_like_soft_requirement(keyword)
            && !hard.iter().any(|skill| {
                search_key(skill) == search_key(keyword)
                    || contains_search_term(skill, keyword)
                    || contains_search_term(keyword, skill)
            })
    });
    keywords = keywords_hors_competences(&keywords, &hard);

    (hard, soft, keywords)
}

fn looks_like_soft_requirement(value: &str) -> bool {
    let key = search_key(value);
    if key.is_empty() {
        return false;
    }
    SOFT_REQUIREMENT_MARKERS.iter().any(|marker| {
        let marker_key = search_key(marker);
        key == marker_key || key.contains(&marker_key) || marker_key.contains(&key)
    })
}

/// Évite de compter deux fois une exigence déjà listée en compétence.
fn keywords_hors_competences(keywords: &[String], skills: &[String]) -> Vec<String> {
    deduplicate_labels(keywords)
        .into_iter()
        .filter(|keyword| {
            let key = search_key(keyword);
            !key.is_empty()
                && !skills.iter().any(|skill| {
                    search_key(skill) == key
                        || contains_search_term(skill, keyword)
                        || contains_search_term(keyword, skill)
                })
        })
        .collect()
}

fn profile_title(profile: &Profile) -> &str {
    profile.identity.title.as_deref().unwrap_or_default()
}

fn annees_experience(profile: &Profile) -> usize {
    let current = chrono::Utc::now().year();
    profile
        .experiences
        .iter()
        .filter_map(|e| {
            year(&e.start_date).map(|start| {
                (year(e.end_date.as_deref().unwrap_or_default()).unwrap_or(current) - start).max(0)
                    as usize
            })
        })
        .sum()
}

/// Corpus candidat : compétences, projets, certifications, langues, expériences, titre.
fn candidate_evidence(profile: &Profile) -> String {
    let skills = profile
        .skills
        .iter()
        .map(|s| {
            format!(
                "{} {}",
                s.name,
                s.description.as_deref().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let experiences = profile
        .experiences
        .iter()
        .map(|e| {
            format!(
                "{} {} {}",
                e.title,
                e.company,
                e.description.as_deref().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let projects = profile
        .projects
        .iter()
        .map(|p| {
            format!(
                "{} {} {}",
                p.name,
                p.technologies.as_deref().unwrap_or_default(),
                p.description.as_deref().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let certifications = profile
        .certifications
        .iter()
        .map(|c| format!("{} {}", c.name, c.issuer.as_deref().unwrap_or_default()))
        .collect::<Vec<_>>()
        .join(" ");
    let languages = profile
        .languages
        .iter()
        .map(|l| format!("{} {}", l.name, l.level))
        .collect::<Vec<_>>()
        .join(" ");
    let education = profile
        .education
        .iter()
        .map(|e| {
            format!(
                "{} {} {}",
                e.degree,
                e.school,
                e.description.as_deref().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    search_key(&format!(
        "{} {} {} {} {} {} {} {}",
        profile.identity.title.as_deref().unwrap_or_default(),
        profile.identity.resume.as_deref().unwrap_or_default(),
        skills,
        experiences,
        projects,
        certifications,
        languages,
        education
    ))
}

/// Correspondance par mot : `"go"` ne match pas `"ongoing"`.
fn contains_term(haystack: &str, needle: &str) -> bool {
    contains_search_term(haystack, needle)
}

fn percentage(count: usize, total: usize) -> Option<u8> {
    (total > 0).then(|| count.saturating_mul(100).saturating_div(total).min(100) as u8)
}

/// Nombre d'années d'expérience exigé, lu dans le texte libre d'une offre.
///
/// Le premier entier du texte n'est pas le bon : « Bac+3, 5 ans d'expérience » donnait 3,
/// et le score d'expérience — 40 % du total profil — s'en trouvait doublé. On retient donc
/// le nombre qui **précède** une mention d'année, et le plus petit lorsqu'une fourchette
/// est annoncée : « 2 à 5 ans » demande deux ans pour candidater, pas cinq.
///
/// Sans mention d'année, on retombe sur le premier entier : une offre qui écrit « 3+ »
/// reste comprise, et l'ancien comportement couvre le reste.
fn first_entier(value: &str) -> usize {
    let normalise = crate::core::utils::text::search_key(value);
    let mots: Vec<&str> = normalise.split_whitespace().collect();
    let exigences: Vec<usize> = mots
        .iter()
        .enumerate()
        .filter(|(_, mot)| matches!(nettoyer(mot), "an" | "ans" | "annee" | "annees"))
        .filter_map(|(index, _)| minimum_avant(&mots[..index]))
        .collect();
    if let Some(minimum) = exigences.into_iter().min() {
        return minimum;
    }
    mots.iter()
        .find_map(|mot| entier_de(mot))
        .unwrap_or_default()
}

/// Plus petit nombre du groupe qui précède immédiatement une mention d'année.
///
/// La remontée s'arrête au premier mot qui n'est ni un nombre entier ni un connecteur :
/// « Bac+3, 5 ans » ne doit pas rendre 3, alors que « 2 à 5 ans » doit rendre 2.
fn minimum_avant(mots: &[&str]) -> Option<usize> {
    let mut trouves = Vec::new();
    for mot in mots.iter().rev() {
        let mot = nettoyer(mot);
        if let Ok(nombre) = mot.parse::<usize>() {
            trouves.push(nombre);
            continue;
        }
        // `search_key` a déjà ramené « à » à « a ».
        if matches!(mot, "a" | "et" | "ou" | "-") {
            continue;
        }
        break;
    }
    trouves.into_iter().min()
}

/// Retire la ponctuation qui colle à un mot (« 5, » → « 5 »).
fn nettoyer(mot: &str) -> &str {
    mot.trim_matches(|c: char| !c.is_alphanumeric())
}

/// Premier entier contenu dans un mot, `None` si le mot n'en porte aucun.
fn entier_de(mot: &str) -> Option<usize> {
    mot.split(|c: char| !c.is_ascii_digit())
        .find(|morceau| !morceau.is_empty())
        .and_then(|morceau| morceau.parse().ok())
}

fn year(value: &str) -> Option<i32> {
    value
        .as_bytes()
        .windows(4)
        .find_map(|v| std::str::from_utf8(v).ok()?.parse().ok())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
