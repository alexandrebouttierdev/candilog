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
    #[serde(default, alias = "ville", deserialize_with = "option_string_lenient")]
    pub city: Option<String>,
    /// Accroche courte, utilisée comme objectif ou titre de CV.
    #[serde(default, alias = "titre", deserialize_with = "option_string_lenient")]
    pub title: Option<String>,
    /// Présentation détaillée du parcours et de l'objectif.
    #[serde(default, deserialize_with = "option_string_lenient")]
    pub resume: Option<String>,
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

/// Profile complet persisté dans la ligne singleton `profil`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Profile {
    #[serde(default, alias = "identite")]
    pub identity: Identity,
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

fn skill_from_value(value: serde_json::Value) -> Option<Skill> {
    match value {
        serde_json::Value::String(name) if !name.trim().is_empty() => Some(Skill { name }),
        serde_json::Value::Object(map) => {
            let name = map
                .get("name")
                .or_else(|| map.get("nom"))
                .map(|item| text_from_value(item.clone()))?;
            if name.trim().is_empty() {
                None
            } else {
                Some(Skill { name })
            }
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
