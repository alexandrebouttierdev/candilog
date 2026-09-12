//! Pipeline lettres naturelles : nettoyage offre, pack d'évidences, grounding prose LLM.

use super::cover_letter::{CoverLetterPlan, GroundedFact, GroundedFactKind};
use super::normalization::contains_search_term;
use super::{search_key, CoverLetterRequest, ValidateAiOutput};
use crate::core::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Brouillon de lettre produit par le LLM (prose), à ancrer ensuite sur le catalogue.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverLetterDraft {
    #[serde(default, alias = "lettre", alias = "text", alias = "body")]
    pub letter: String,
}

impl ValidateAiOutput for CoverLetterDraft {
    fn validate_ai_output(&self) -> AppResult<()> {
        if self.letter.chars().count() > 12_000 {
            return Err(AppError::Provider(
                "La lettre proposée par l'IA est trop longue.".into(),
            ));
        }
        super::validate_structured_size(self)
    }
}

/// Retire codes d'annonce, slogans et process RH — générique, sans logique métier.
#[must_use]
pub fn clean_offer_context(raw: &str) -> String {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !is_offer_noise_line(line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Clé de matching lettres : accents + ponctuation (apostrophes, « Réf. ») → espaces.
fn letter_match_key(value: &str) -> String {
    let stripped: String = value
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect();
    search_key(&stripped)
}

fn is_offer_noise_line(line: &str) -> bool {
    let key = letter_match_key(line);
    const MARKERS: &[&str] = &[
        "code rec",
        "code offre",
        "reference",
        "attendez vous",
        "process de recrutement",
        "notre process",
        "processus de recrutement",
        "echange telephonique",
        "entretien rh",
        "mentions legales",
        "egalite des chances",
        "donnees personnelles",
        "rgpd",
    ];
    let looks_like_ref = key.starts_with("ref ")
        || key.starts_with("ref ")
        || key.starts_with("reference")
        || key.starts_with("code offre")
        || key.starts_with("code rec");
    looks_like_ref || MARKERS.iter().any(|m| key.contains(&letter_match_key(m)))
}

/// Version courte d'un fait pour la rédaction (pas de dump de puces CV).
#[must_use]
pub fn shorten_evidence_text(kind: GroundedFactKind, text: &str) -> String {
    let flat = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| l.trim_start_matches(['•', '-', '·', '*']).trim())
        .collect::<Vec<_>>()
        .join(" ");
    let max = match kind {
        GroundedFactKind::Experience | GroundedFactKind::Project => 220,
        GroundedFactKind::Summary => 260,
        _ => 120,
    };
    truncate_at_sentence(&flat, max)
}

fn truncate_at_sentence(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_owned();
    }
    let mut cut = trimmed
        .char_indices()
        .nth(max_chars)
        .map(|(i, _)| i)
        .unwrap_or(trimmed.len());
    if let Some(rel) = trimmed[..cut].rfind(['.', '!', '?']) {
        let ch = trimmed[rel..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        cut = rel + ch;
    } else if let Some(rel) = trimmed[..cut].rfind(' ') {
        cut = rel;
    }
    format!("{}…", trimmed[..cut].trim_end())
}

fn fact_kind_priority(kind: GroundedFactKind) -> u8 {
    match kind {
        GroundedFactKind::Experience => 0,
        GroundedFactKind::Summary => 1,
        GroundedFactKind::Skill => 2,
        GroundedFactKind::Project => 3,
        GroundedFactKind::Education => 4,
        GroundedFactKind::Certification => 5,
    }
}

/// Construit le pack d'évidences (textes courts) à partir du plan LLM.
pub fn resolve_letter_evidence(
    catalog: &[GroundedFact],
    plan: &CoverLetterPlan,
    fact_limit: usize,
) -> AppResult<Vec<GroundedFact>> {
    let by_id: HashMap<&str, &GroundedFact> = catalog
        .iter()
        .map(|fact| (fact.id.as_str(), fact))
        .collect();
    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    for id in &plan.selected_fact_ids {
        let fact = by_id.get(id.as_str()).ok_or_else(|| {
            AppError::Provider("La réponse IA référence un fait inconnu du profil.".into())
        })?;
        if seen.insert(id.as_str()) && selected.len() < fact_limit {
            selected.push(GroundedFact {
                id: fact.id.clone(),
                kind: fact.kind,
                text: shorten_evidence_text(fact.kind, &fact.text),
            });
        }
    }
    if selected.len() < fact_limit {
        let mut ranked: Vec<&GroundedFact> = catalog.iter().collect();
        ranked.sort_by_key(|fact| fact_kind_priority(fact.kind));
        for fact in ranked {
            if selected.len() >= fact_limit {
                break;
            }
            if seen.insert(fact.id.as_str()) {
                selected.push(GroundedFact {
                    id: fact.id.clone(),
                    kind: fact.kind,
                    text: shorten_evidence_text(fact.kind, &fact.text),
                });
            }
        }
    }
    selected.sort_by_key(|fact| fact_kind_priority(fact.kind));
    Ok(selected)
}

pub fn fact_limit_for_request(request: &CoverLetterRequest) -> AppResult<usize> {
    match request.length.as_deref().unwrap_or("medium") {
        "short" => Ok(1),
        "medium" => Ok(2),
        "long" => Ok(3),
        _ => Err(AppError::Validation(
            "La longueur de lettre demandée n'est pas prise en charge.".into(),
        )),
    }
}

/// Ancre la prose LLM : retire les phrases assertives sans preuve dans le pack / brief.
#[must_use]
pub fn ground_cover_letter(
    letter: &str,
    evidence: &[GroundedFact],
    company: &str,
    job_title: &str,
    cleaned_offer: &str,
) -> String {
    let corpus = letter_match_key(&format!(
        "{} {} {} {}",
        evidence
            .iter()
            .map(|e| e.text.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        company,
        job_title,
        cleaned_offer
    ));
    let mut kept = Vec::new();
    for block in letter.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        if is_letter_formula(block) || sentence_supported(block, &corpus) {
            kept.push(block.to_owned());
        }
    }
    if kept.is_empty() {
        return letter.trim().to_owned();
    }
    kept.join("\n\n")
}

fn is_letter_formula(block: &str) -> bool {
    let key = search_key(block);
    key.starts_with("madame monsieur")
        || key.starts_with("bonjour")
        || key.contains("cordialement")
        || key.contains("salutations distinguees")
        || key.contains("je reste a votre disposition")
        || key.contains("je serais heureux")
        || key.contains("je serais ravi")
}

fn sentence_supported(block: &str, corpus: &str) -> bool {
    let tokens: Vec<String> = letter_match_key(block)
        .split_whitespace()
        .filter(|t| t.len() >= 5)
        .filter(|t| {
            !matches!(
                *t,
                "madame"
                    | "monsieur"
                    | "poste"
                    | "entreprise"
                    | "candidature"
                    | "parcours"
                    | "experience"
                    | "souhaiterais"
                    | "voudrais"
                    | "notamment"
                    | "egalement"
                    | "toujours"
                    | "jamais"
                    | "familiarise"
                    | "accompagne"
            )
        })
        .map(str::to_owned)
        .collect();
    if tokens.is_empty() {
        return true;
    }
    let hits = tokens
        .iter()
        .filter(|t| {
            // corpus déjà en letter_match_key : comparaison directe ou sous-chaîne bornée.
            corpus.split_whitespace().any(|w| w == *t) || contains_search_term(corpus, t)
        })
        .count();
    hits * 2 >= tokens.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_offer_retire_code_rec_et_slogans() {
        let raw = "Poste développeur\nCode REC: 12345\nQu'attendez-vous pour postuler ?\nReact et TypeScript\nNotre process de recrutement\nUn bref échange téléphonique";
        let cleaned = clean_offer_context(raw);
        assert!(cleaned.contains("Poste développeur"));
        assert!(cleaned.contains("React"));
        assert!(!cleaned.contains("Code REC"));
        assert!(!search_key(&cleaned).contains("attendez"));
        assert!(!search_key(&cleaned).contains("process de recrutement"));
        assert!(!search_key(&cleaned).contains("echange telephonique"));
    }

    #[test]
    fn shorten_experience_ne_dump_pas_toutes_les_puces() {
        let long = "Dev chez Nova — APIs.\n• Rust\n• PostgreSQL\n• CI/CD\n• Kubernetes\n• Observabilité\n• Mentoring";
        let short = shorten_evidence_text(GroundedFactKind::Experience, long);
        assert!(short.chars().count() <= 230, "{short}");
        assert!(short.contains("Nova"));
    }

    #[test]
    fn ground_retire_phrase_inventee() {
        let evidence = vec![GroundedFact {
            id: "experience:0".into(),
            kind: GroundedFactKind::Experience,
            text: "Technicien logistique chez Nova — préparation de commandes.".into(),
        }];
        let letter = "Madame, Monsieur,\n\nJ'ai une expérience en préparation de commandes chez Nova.\n\nJe possède le CACES 3 et un permis poids lourd depuis dix ans.\n\nCordialement,";
        let grounded = ground_cover_letter(letter, &evidence, "Acme", "Préparateur", "");
        assert!(
            grounded.contains("préparation") || grounded.contains("Nova"),
            "{grounded}"
        );
        assert!(!grounded.contains("CACES"), "{grounded}");
        assert!(!grounded.contains("poids lourd"), "{grounded}");
        assert!(grounded.contains("Cordialement"));
    }

    #[test]
    fn resolve_refuse_id_inconnu() {
        let catalog = vec![GroundedFact {
            id: "skill:0".into(),
            kind: GroundedFactKind::Skill,
            text: "Excel".into(),
        }];
        let plan = CoverLetterPlan {
            selected_fact_ids: vec!["skill:99".into()],
            motivation_keywords: vec![],
        };
        assert!(resolve_letter_evidence(&catalog, &plan, 2).is_err());
    }

    #[test]
    fn clean_offre_marketing_lourde_garde_le_fond() {
        let raw = "Commercial BtoB
Réf. COM-99
Code offre: ZX
Relation client et négociation
Égalité des chances
RGPD";
        let cleaned = clean_offer_context(raw);
        assert!(
            cleaned.contains("Commercial") || cleaned.contains("Relation"),
            "{cleaned}"
        );
        assert!(!cleaned.contains("COM-99"), "{cleaned}");
        assert!(!cleaned.contains("RGPD"), "{cleaned}");
    }

    #[test]
    fn ground_garde_faits_sante_sans_inventer_ide() {
        let evidence = vec![GroundedFact {
            id: "experience:0".into(),
            kind: GroundedFactKind::Experience,
            text: "Aide-soignant chez Clinique Sud — accompagnement des patients.".into(),
        }];
        let letter = "Madame, Monsieur,

Mon parcours d'aide-soignant chez Clinique Sud m'a familiarisé avec l'accompagnement des patients.

Je suis infirmier diplômé d'État depuis 2018.

Cordialement,";
        let grounded = ground_cover_letter(letter, &evidence, "CHU", "Aide-soignant", "");
        assert!(
            grounded.contains("aide-soignant") || grounded.contains("Clinique"),
            "{grounded}"
        );
        assert!(!grounded.contains("infirmier"), "{grounded}");
        assert!(!grounded.contains("2018"), "{grounded}");
    }
}
