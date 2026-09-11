//! Entités du profil exposées à React.

use serde::{Deserialize, Deserializer, Serialize};

/// Coordonnées et objectif professionnel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Identity {
    #[serde(default, alias = "prenom", deserialize_with = "string_lenient")]
    pub first_name: String,
    #[serde(default, alias = "nom", deserialize_with = "string_lenient")]
    pub name: String,
    #[serde(default, deserialize_with = "string_lenient")]
    pub email: String,
    #[serde(
        default,
        alias = "telephone",
        deserialize_with = "option_string_lenient"
    )]
    pub phone: Option<String>,
    #[serde(default, alias = "adresse", deserialize_with = "option_string_lenient")]
    pub address: Option<String>,
    #[serde(default, alias = "ville", deserialize_with = "option_string_lenient")]
    pub city: Option<String>,
    /// Accroche courte, utilisée comme objectif ou titre de CV.
    #[serde(default, alias = "titre", deserialize_with = "option_string_lenient")]
    pub title: Option<String>,
    /// Présentation / résumé du profil CV — facultatif.
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub resume: Option<String>,
    /// Date de naissance telle qu'écrite sur le CV — facultatif.
    #[serde(
        default,
        alias = "dateNaissance",
        alias = "date_naissance",
        deserialize_with = "option_string_lenient"
    )]
    pub birth_date: Option<String>,
    /// Âge déclaré sur le CV — facultatif, non recalculé automatiquement.
    #[serde(default, deserialize_with = "option_u8_lenient")]
    #[ts(type = "number | null")]
    pub age: Option<u8>,
    /// Disponibilité (ex. « Sous 1 mois ») — facultatif.
    #[serde(
        default,
        alias = "disponibilite",
        deserialize_with = "option_string_lenient"
    )]
    pub availability: Option<String>,
    /// Contrats recherchés (ex. « CDI • Freelance ») — facultatif.
    #[serde(
        default,
        alias = "contrats",
        deserialize_with = "option_string_lenient"
    )]
    pub desired_contracts: Option<String>,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub linkedin: Option<String>,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub github: Option<String>,
    #[serde(
        default,
        alias = "siteWeb",
        alias = "site_web",
        deserialize_with = "option_string_lenient"
    )]
    pub website: Option<String>,
}

/// Expérience professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Experience {
    #[serde(default, alias = "intitule", deserialize_with = "string_lenient")]
    pub title: String,
    #[serde(default, alias = "entreprise", deserialize_with = "string_lenient")]
    pub company: String,
    #[serde(default, alias = "lieu", deserialize_with = "option_string_lenient")]
    pub location: Option<String>,
    #[serde(default, deserialize_with = "string_lenient")]
    pub start_date: String,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub end_date: Option<String>,
    #[serde(default, alias = "posteActuel", alias = "poste_actuel")]
    pub current: bool,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub description: Option<String>,
}

/// Compétence professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Skill {
    #[serde(default, alias = "nom", deserialize_with = "string_lenient")]
    pub name: String,
}

/// Education académique ou professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Education {
    #[serde(default, alias = "diplome", deserialize_with = "string_lenient")]
    pub degree: String,
    #[serde(default, alias = "etablissement", deserialize_with = "string_lenient")]
    pub school: String,
    #[serde(default, alias = "lieu", deserialize_with = "option_string_lenient")]
    pub location: Option<String>,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub start_date: Option<String>,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub end_date: Option<String>,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub description: Option<String>,
}

/// Language parlée et niveau associé.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Language {
    #[serde(default, alias = "nom", deserialize_with = "string_lenient")]
    pub name: String,
    #[serde(default, alias = "niveau", deserialize_with = "string_lenient")]
    pub level: String,
}

/// Project personnel ou professionnel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Project {
    #[serde(default, alias = "nom", deserialize_with = "string_lenient")]
    pub name: String,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub url: Option<String>,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub technologies: Option<String>,
}

/// Certification obtenue.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Certification {
    #[serde(default, alias = "nom", deserialize_with = "string_lenient")]
    pub name: String,
    #[serde(
        default,
        alias = "organisme",
        deserialize_with = "option_string_lenient"
    )]
    pub issuer: Option<String>,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub date: Option<String>,
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub url: Option<String>,
}

/// Centre d'intérêt déclaré sur le CV — facultatif.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Interest {
    #[serde(default, alias = "nom", deserialize_with = "string_lenient")]
    pub name: String,
}

/// Profile complet persisté dans la ligne singleton `profil`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Profile {
    #[serde(default, alias = "identite")]
    pub identity: Identity,
    /// Nom du fichier de la photo dans le dossier de données, jamais un chemin absolu.
    ///
    /// Hors du formulaire : `profile_save` ne peut pas y toucher, seules les commandes
    /// dédiées la posent ou la retirent. Un écran qui n'affiche pas la photo ne risque donc
    /// pas de l'effacer en enregistrant le reste du profil.
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub photo: Option<String>,
    #[serde(default)]
    pub experiences: Vec<Experience>,
    #[serde(default, alias = "competences", deserialize_with = "skills_lenient")]
    pub skills: Vec<Skill>,
    #[serde(default, alias = "formations")]
    pub education: Vec<Education>,
    #[serde(default, alias = "langues")]
    pub languages: Vec<Language>,
    #[serde(default, alias = "projets")]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub certifications: Vec<Certification>,
    #[serde(
        default,
        alias = "centresInterets",
        alias = "centres_interets",
        deserialize_with = "interests_lenient"
    )]
    pub interests: Vec<Interest>,
}

/// Payload utile de l'écran Profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ProfilePayload {
    pub profile: Profile,
    /// Score de complétion entre 0 et 100.
    #[ts(type = "number")]
    pub completion: u8,
    /// Sections encore absentes, dans l'ordre utile à l'utilisateur.
    pub incomplete_sections: Vec<String>,
    /// Timestamp du dernier enregistrement, absent pour un profil neuf.
    pub updated_at: Option<String>,
}

/// Les modèles LLM envoient souvent une liste de puces là où le profil attend une chaîne.
fn string_lenient<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(text_from_value(serde_json::Value::deserialize(
        deserializer,
    )?))
}

fn option_string_lenient<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let text = text_from_value(serde_json::Value::deserialize(deserializer)?);
    Ok(if text.trim().is_empty() {
        None
    } else {
        Some(text)
    })
}

fn skills_lenient<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Skill>, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Array(items) => items.into_iter().filter_map(skill_from_value).collect(),
        serde_json::Value::String(name) if !name.trim().is_empty() => vec![Skill { name }],
        _ => Vec::new(),
    })
}

fn interests_lenient<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Interest>, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Array(items) => {
            items.into_iter().filter_map(interest_from_value).collect()
        }
        serde_json::Value::String(name) if !name.trim().is_empty() => vec![Interest { name }],
        _ => Vec::new(),
    })
}

fn skill_from_value(value: serde_json::Value) -> Option<Skill> {
    named_item(value).map(|name| Skill { name })
}

fn interest_from_value(value: serde_json::Value) -> Option<Interest> {
    named_item(value).map(|name| Interest { name })
}

fn named_item(value: serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(name) if !name.trim().is_empty() => Some(name),
        serde_json::Value::Object(map) => {
            let name = map
                .get("name")
                .or_else(|| map.get("nom"))
                .map(|item| text_from_value(item.clone()))?;
            if name.trim().is_empty() {
                None
            } else {
                Some(name)
            }
        }
        _ => None,
    }
}

fn option_u8_lenient<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u8>, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(age_from_value(value))
}

fn age_from_value(value: serde_json::Value) -> Option<u8> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Number(number) => number.as_u64().and_then(|n| u8::try_from(n).ok()),
        serde_json::Value::String(text) => {
            let digits: String = text.chars().filter(|ch| ch.is_ascii_digit()).collect();
            if digits.is_empty() {
                return None;
            }
            digits
                .parse::<u8>()
                .ok()
                .filter(|age| (1..=120).contains(age))
        }
        _ => None,
    }
}

fn text_from_value(value: serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text,
        serde_json::Value::Number(number) => number.to_string(),
        serde_json::Value::Bool(flag) => flag.to_string(),
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(text_from_value)
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        serde_json::Value::Object(map) => map
            .into_values()
            .map(text_from_value)
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" — "),
        serde_json::Value::Null => String::new(),
    }
}
