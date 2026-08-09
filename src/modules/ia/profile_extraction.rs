//! Extraction structurée d'un profil à partir du texte brut d'un CV.
//!
//! Les modèles compacts renvoient un `JSON` approximatif (scalaires là où on
//! attend des chaînes, objet au lieu d'un tableau, clés absentes…). Ce module
//! définit un miroir **tolérant** de [`Profile`](crate::shared::profile::Profile),
//! puis le **normalise** (dates, niveaux de langue, noms de compétences) en un
//! `Profile` propre via [`From`].

use crate::shared::profile::{
    Certification, Education, Experience, Language, PersonalInfo, Profile, Project, Skill,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// Désérialise une chaîne en tolérant les scalaires non-chaîne du `LLM`.
fn de_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(value_to_string(Value::deserialize(deserializer)?).unwrap_or_default())
}

/// Désérialise une chaîne optionnelle ; `None` si vide, `null`, ou non scalaire.
fn de_opt_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = value_to_string(Value::deserialize(deserializer)?);
    Ok(value.filter(|s| !s.trim().is_empty()))
}

/// Désérialise un booléen en tolérant `"true"`/`"oui"`/`1` produits par le `LLM`.
fn de_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Bool(b) => b,
        Value::Number(n) => n.as_i64().unwrap_or_default() != 0,
        Value::String(s) => matches!(
            s.trim().to_lowercase().as_str(),
            "true" | "oui" | "yes" | "1"
        ),
        _ => false,
    })
}

/// Désérialise un tableau tolérant : accepte un tableau, `null` (→ vide), ou un
/// élément isolé (→ tableau à un élément). Les éléments illisibles sont ignorés.
fn de_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|item| serde_json::from_value(item).ok())
            .collect(),
        Value::Null => Vec::new(),
        other => serde_json::from_value(other).ok().into_iter().collect(),
    })
}

/// Convertit une valeur `JSON` scalaire (ou objet/tableau simple) en chaîne lisible.
fn value_to_string(value: Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Array(items) => {
            let joined = items
                .into_iter()
                .filter_map(value_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            (!joined.is_empty()).then_some(joined)
        }
        Value::Object(fields) => {
            let joined = fields
                .into_values()
                .filter_map(value_to_string)
                .collect::<Vec<_>>()
                .join(" — ");
            (!joined.is_empty()).then_some(joined)
        }
        Value::Null => None,
    }
}

/// Informations personnelles extraites (miroir tolérant de `PersonalInfo`).
///
/// `pub` car extraite par un appel `LLM` spécialisé dédié (voir `CvEngine::extract_profile`).
#[derive(Debug, Default, Deserialize)]
pub struct ExtractedPersonal {
    #[serde(default, deserialize_with = "de_string")]
    first_name: String,
    #[serde(default, deserialize_with = "de_string")]
    last_name: String,
    #[serde(default, deserialize_with = "de_string")]
    email: String,
    #[serde(default, deserialize_with = "de_opt_string")]
    phone: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    city: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    headline: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    summary: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    linkedin: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    github: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    website: Option<String>,
}

/// Expérience extraite (miroir tolérant de `Experience`).
#[derive(Debug, Default, Deserialize)]
struct ExtractedExperience {
    #[serde(default, deserialize_with = "de_string")]
    title: String,
    #[serde(default, deserialize_with = "de_string")]
    company: String,
    #[serde(default, deserialize_with = "de_opt_string")]
    location: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    start_date: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    end_date: Option<String>,
    #[serde(default, deserialize_with = "de_bool")]
    current: bool,
    #[serde(default, deserialize_with = "de_opt_string")]
    description: Option<String>,
}

/// Compétence extraite (miroir tolérant de `Skill`).
#[derive(Debug, Default, Deserialize)]
struct ExtractedSkill {
    #[serde(default, deserialize_with = "de_string")]
    name: String,
}

/// Formation extraite (miroir tolérant de `Education`).
#[derive(Debug, Default, Deserialize)]
struct ExtractedEducation {
    #[serde(default, deserialize_with = "de_string")]
    degree: String,
    #[serde(default, deserialize_with = "de_string")]
    school: String,
    #[serde(default, deserialize_with = "de_opt_string")]
    location: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    start_date: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    end_date: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    description: Option<String>,
}

/// Langue extraite (miroir tolérant de `Language`).
#[derive(Debug, Default, Deserialize)]
struct ExtractedLanguage {
    #[serde(default, deserialize_with = "de_string")]
    name: String,
    #[serde(default, deserialize_with = "de_string")]
    level: String,
}

/// Projet extrait (miroir tolérant de `Project`).
#[derive(Debug, Default, Deserialize)]
struct ExtractedProject {
    #[serde(default, deserialize_with = "de_string")]
    name: String,
    #[serde(default, deserialize_with = "de_opt_string")]
    description: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    url: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    technologies: Option<String>,
}

/// Certification extraite (miroir tolérant de `Certification`).
#[derive(Debug, Default, Deserialize)]
struct ExtractedCertification {
    #[serde(default, deserialize_with = "de_string")]
    name: String,
    #[serde(default, deserialize_with = "de_opt_string")]
    issuer: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    date: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    url: Option<String>,
}

/// Profil extrait d'un CV par le `LLM`, avant normalisation.
///
/// Structurellement identique à [`Profile`](crate::shared::profile::Profile),
/// mais désérialisé de façon tolérante. Utiliser [`Profile::from`] pour obtenir
/// un profil propre et normalisé.
#[derive(Debug, Default, Deserialize)]
pub struct ExtractedProfile {
    #[serde(default)]
    personal: ExtractedPersonal,
    #[serde(default, deserialize_with = "de_vec")]
    experiences: Vec<ExtractedExperience>,
    #[serde(default, deserialize_with = "de_vec")]
    skills: Vec<ExtractedSkill>,
    #[serde(default, deserialize_with = "de_vec")]
    education: Vec<ExtractedEducation>,
    #[serde(default, deserialize_with = "de_vec")]
    languages: Vec<ExtractedLanguage>,
    #[serde(default, deserialize_with = "de_vec")]
    projects: Vec<ExtractedProject>,
    #[serde(default, deserialize_with = "de_vec")]
    certifications: Vec<ExtractedCertification>,
}

/// Parcours extrait (expériences + formations) — sortie d'un appel `LLM` spécialisé.
#[derive(Debug, Default, Deserialize)]
pub struct ExtractedHistory {
    #[serde(default, deserialize_with = "de_vec")]
    experiences: Vec<ExtractedExperience>,
    #[serde(default, deserialize_with = "de_vec")]
    education: Vec<ExtractedEducation>,
}

/// Compétences + langues extraites — sortie d'un appel `LLM` spécialisé.
#[derive(Debug, Default, Deserialize)]
pub struct ExtractedSkillsLangs {
    #[serde(default, deserialize_with = "de_vec")]
    skills: Vec<ExtractedSkill>,
    #[serde(default, deserialize_with = "de_vec")]
    languages: Vec<ExtractedLanguage>,
}

/// Projets + certifications extraits — sortie d'un appel `LLM` spécialisé.
#[derive(Debug, Default, Deserialize)]
pub struct ExtractedPortfolio {
    #[serde(default, deserialize_with = "de_vec")]
    projects: Vec<ExtractedProject>,
    #[serde(default, deserialize_with = "de_vec")]
    certifications: Vec<ExtractedCertification>,
}

/// Raccourci de schéma : chaîne `JSON` obligatoire.
fn schema_string() -> serde_json::Value {
    serde_json::json!({"type": "string"})
}

/// Raccourci de schéma : chaîne `JSON` optionnelle (`null` si absente du CV).
fn schema_opt_string() -> serde_json::Value {
    serde_json::json!({"type": ["string", "null"]})
}

/// Schéma `JSON` de l'identité ([`ExtractedPersonal`]) — décodage contraint.
#[must_use]
pub fn identity_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "first_name": schema_string(),
            "last_name": schema_string(),
            "email": schema_string(),
            "phone": schema_opt_string(),
            "city": schema_opt_string(),
            "headline": schema_opt_string(),
            "summary": schema_opt_string(),
            "linkedin": schema_opt_string(),
            "github": schema_opt_string(),
            "website": schema_opt_string(),
        },
        "required": ["first_name", "last_name", "email", "phone", "city", "headline", "summary", "linkedin", "github", "website"],
    })
}

/// Schéma `JSON` du parcours ([`ExtractedHistory`]) — décodage contraint.
#[must_use]
pub fn history_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "experiences": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": schema_string(),
                        "company": schema_string(),
                        "location": schema_opt_string(),
                        "start_date": schema_opt_string(),
                        "end_date": schema_opt_string(),
                        "current": {"type": "boolean"},
                        "description": schema_opt_string(),
                    },
                    "required": ["title", "company", "location", "start_date", "end_date", "current", "description"],
                },
            },
            "education": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "degree": schema_string(),
                        "school": schema_string(),
                        "location": schema_opt_string(),
                        "start_date": schema_opt_string(),
                        "end_date": schema_opt_string(),
                        "description": schema_opt_string(),
                    },
                    "required": ["degree", "school", "location", "start_date", "end_date", "description"],
                },
            },
        },
        "required": ["experiences", "education"],
    })
}

/// Schéma `JSON` des compétences et langues ([`ExtractedSkillsLangs`]) — décodage contraint.
#[must_use]
pub fn skills_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "skills": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {"name": schema_string()},
                    "required": ["name"],
                },
            },
            "languages": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {"name": schema_string(), "level": schema_string()},
                    "required": ["name", "level"],
                },
            },
        },
        "required": ["skills", "languages"],
    })
}

/// Schéma `JSON` des projets et certifications ([`ExtractedPortfolio`]) — décodage contraint.
#[must_use]
pub fn portfolio_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "projects": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": schema_string(),
                        "description": schema_opt_string(),
                        "url": schema_opt_string(),
                        "technologies": schema_opt_string(),
                    },
                    "required": ["name", "description", "url", "technologies"],
                },
            },
            "certifications": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": schema_string(),
                        "issuer": schema_opt_string(),
                        "date": schema_opt_string(),
                        "url": schema_opt_string(),
                    },
                    "required": ["name", "issuer", "date", "url"],
                },
            },
        },
        "required": ["projects", "certifications"],
    })
}

impl ExtractedProfile {
    /// Assemble un profil extrait à partir des sorties des appels `LLM` spécialisés.
    ///
    /// Découper l'extraction en étapes ciblées (identité, parcours, compétences/langues,
    /// projets/certifications) donne des schémas plus simples et bien plus fiables sur
    /// un petit modèle qu'une extraction monolithique.
    #[must_use]
    pub fn from_sections(
        personal: ExtractedPersonal,
        history: ExtractedHistory,
        lists: ExtractedSkillsLangs,
        portfolio: ExtractedPortfolio,
    ) -> Self {
        Self {
            personal,
            experiences: history.experiences,
            skills: lists.skills,
            education: history.education,
            languages: lists.languages,
            projects: portfolio.projects,
            certifications: portfolio.certifications,
        }
    }
}

/// Réduit une chaîne à sa forme normalisée : espaces de tête/queue retirés et
/// suites d'espaces internes compactées en un seul espace.
fn collapse_whitespace(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalise `raw` en `None` s'il est vide après compactage, sinon `Some`.
fn clean_opt(raw: Option<String>) -> Option<String> {
    raw.map(|s| collapse_whitespace(&s))
        .filter(|s| !s.is_empty())
}

/// Normalise un niveau de langue en une échelle stable en français.
///
/// Reconnaît les mentions courantes (natif, bilingue, courant, intermédiaire,
/// débutant) et le référentiel `CECR` (`A1`–`C2`). Toute valeur non reconnue est
/// conservée telle quelle (compactée), pour ne pas perdre d'information.
fn normalize_language_level(raw: &str) -> String {
    let lower = raw.to_lowercase();
    let has = |needles: &[&str]| needles.iter().any(|n| lower.contains(n));
    if has(&["natif", "native", "maternel", "bilingue", "mother"]) {
        "Bilingue / natif".to_string()
    } else if has(&["courant", "fluent", "c1", "c2", "avancé", "advanced"]) {
        "Courant".to_string()
    } else if has(&["intermédiaire", "intermediate", "moyen", "b1", "b2"]) {
        "Intermédiaire".to_string()
    } else if has(&[
        "débutant",
        "beginner",
        "notions",
        "scolaire",
        "a1",
        "a2",
        "basic",
    ]) {
        "Débutant".to_string()
    } else {
        collapse_whitespace(raw)
    }
}

/// Préfixes de noms de mois (français/anglais) reconnus, indexés de janvier (0) à décembre (11).
const MONTH_PREFIXES: [&[&str]; 12] = [
    &["janv", "jan"],
    &["févr", "fev", "feb"],
    &["mars", "mar"],
    &["avr", "apr"],
    &["mai", "may"],
    &["juin", "jun"],
    &["juil", "jul"],
    &["août", "aout", "aug"],
    &["sept", "sep"],
    &["oct"],
    &["nov"],
    &["déc", "dec"],
];

/// Numéro de mois (1–12) reconnu à partir d'un jeton chiffré ou d'un nom de mois
/// français/anglais (abréviations incluses).
fn month_from_token(token: &str) -> Option<u8> {
    if let Ok(n) = token.parse::<u8>() {
        return (1..=12).contains(&n).then_some(n);
    }
    let t = token.to_lowercase();
    MONTH_PREFIXES
        .iter()
        .position(|prefixes| prefixes.iter().any(|p| t.starts_with(p)))
        .map(|i| u8::try_from(i).unwrap_or_default() + 1)
}

/// Normalise une date libre de CV vers `YYYY-MM` (ou `YYYY`) quand c'est possible.
///
/// Repère la première année plausible (1900–2100) et un éventuel mois (chiffré ou
/// nommé) parmi les jetons. Les mentions « présent / en cours / actuel » et les
/// valeurs non datées sont renvoyées compactées telles quelles.
fn normalize_date(raw: &str) -> Option<String> {
    let cleaned = collapse_whitespace(raw);
    if cleaned.is_empty() {
        return None;
    }
    let lower = cleaned.to_lowercase();
    if [
        "présent", "present", "en cours", "actuel", "aujourd", "now", "current",
    ]
    .iter()
    .any(|m| lower.contains(m))
    {
        return Some("Présent".to_string());
    }
    let tokens: Vec<&str> = cleaned
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    let year = tokens
        .iter()
        .find_map(|t| t.parse::<u16>().ok().filter(|y| (1900..=2100).contains(y)));
    let Some(year) = year else {
        return Some(cleaned);
    };
    let month = tokens
        .iter()
        .filter(|t| t.parse::<u16>() != Ok(year))
        .find_map(|t| month_from_token(t));
    Some(match month {
        Some(m) => format!("{year}-{m:02}"),
        None => year.to_string(),
    })
}

impl From<ExtractedPersonal> for PersonalInfo {
    fn from(p: ExtractedPersonal) -> Self {
        Self {
            first_name: collapse_whitespace(&p.first_name),
            last_name: collapse_whitespace(&p.last_name),
            email: collapse_whitespace(&p.email),
            phone: clean_opt(p.phone),
            city: clean_opt(p.city),
            headline: clean_opt(p.headline),
            summary: p
                .summary
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            linkedin: clean_opt(p.linkedin),
            github: clean_opt(p.github),
            website: clean_opt(p.website),
        }
    }
}

/// Collecte et normalise les expériences extraites en [`Experience`].
fn collect_experiences(extracted: Vec<ExtractedExperience>) -> Vec<Experience> {
    extracted
        .into_iter()
        .filter(|e| !e.title.trim().is_empty() || !e.company.trim().is_empty())
        .map(|e| Experience {
            title: collapse_whitespace(&e.title),
            company: collapse_whitespace(&e.company),
            location: clean_opt(e.location),
            start_date: normalize_date(e.start_date.as_deref().unwrap_or_default())
                .unwrap_or_default(),
            end_date: if e.current {
                None
            } else {
                e.end_date.as_deref().and_then(normalize_date)
            },
            current: e.current,
            description: e
                .description
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        })
        .collect()
}

/// Collecte et dédoublonne les compétences extraites en [`Skill`].
fn collect_skills(extracted: Vec<ExtractedSkill>) -> Vec<Skill> {
    let mut seen_skills = std::collections::HashSet::new();
    extracted
        .into_iter()
        .map(|s| Skill {
            name: collapse_whitespace(&s.name),
        })
        .filter(|s| !s.name.is_empty() && seen_skills.insert(s.name.to_lowercase()))
        .collect()
}

/// Collecte et normalise les formations extraites en [`Education`].
fn collect_education(extracted: Vec<ExtractedEducation>) -> Vec<Education> {
    extracted
        .into_iter()
        .filter(|e| !e.degree.trim().is_empty() || !e.school.trim().is_empty())
        .map(|e| Education {
            degree: collapse_whitespace(&e.degree),
            school: collapse_whitespace(&e.school),
            location: clean_opt(e.location),
            start_date: e.start_date.as_deref().and_then(normalize_date),
            end_date: e.end_date.as_deref().and_then(normalize_date),
            description: e
                .description
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        })
        .collect()
}

/// Collecte et normalise les langues extraites en [`Language`].
fn collect_languages(extracted: Vec<ExtractedLanguage>) -> Vec<Language> {
    extracted
        .into_iter()
        .map(|l| Language {
            name: collapse_whitespace(&l.name),
            level: normalize_language_level(&l.level),
        })
        .filter(|l| !l.name.is_empty())
        .collect()
}

/// Collecte et normalise les projets extraits en [`Project`].
fn collect_projects(extracted: Vec<ExtractedProject>) -> Vec<Project> {
    extracted
        .into_iter()
        .filter(|p| !p.name.trim().is_empty())
        .map(|p| Project {
            name: collapse_whitespace(&p.name),
            description: p
                .description
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            url: clean_opt(p.url),
            technologies: clean_opt(p.technologies),
        })
        .collect()
}

/// Collecte et normalise les certifications extraites en [`Certification`].
fn collect_certifications(extracted: Vec<ExtractedCertification>) -> Vec<Certification> {
    extracted
        .into_iter()
        .filter(|c| !c.name.trim().is_empty())
        .map(|c| Certification {
            name: collapse_whitespace(&c.name),
            issuer: clean_opt(c.issuer),
            date: c.date.as_deref().and_then(normalize_date),
            url: clean_opt(c.url),
        })
        .collect()
}

impl From<ExtractedProfile> for Profile {
    fn from(extracted: ExtractedProfile) -> Self {
        Self {
            personal: extracted.personal.into(),
            experiences: collect_experiences(extracted.experiences),
            skills: collect_skills(extracted.skills),
            education: collect_education(extracted.education),
            languages: collect_languages(extracted.languages),
            projects: collect_projects(extracted.projects),
            certifications: collect_certifications(extracted.certifications),
        }
    }
}

/// Indique qu'un profil ne contient aucune donnée exploitable (ni identité, ni
/// aucune section renseignée) — sert à détecter un CV illisible par le `LLM`.
#[must_use]
pub fn profile_is_empty(profile: &Profile) -> bool {
    let p = &profile.personal;
    p.first_name.trim().is_empty()
        && p.last_name.trim().is_empty()
        && p.email.trim().is_empty()
        && profile.experiences.is_empty()
        && profile.skills.is_empty()
        && profile.education.is_empty()
        && profile.languages.is_empty()
        && profile.projects.is_empty()
        && profile.certifications.is_empty()
}

#[cfg(test)]
#[path = "tests/profile_extraction/mod.rs"]
mod tests;
