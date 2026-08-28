//! Entités du profil exposées à React.

use serde::{Deserialize, Serialize};

/// Coordonnées et objectif professionnel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Identity {
    pub first_name: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub city: Option<String>,
    /// Accroche courte, utilisée comme objectif ou titre de CV.
    pub title: Option<String>,
    /// Présentation détaillée du parcours et de l'objectif.
    pub resume: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub website: Option<String>,
}

/// Expérience professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Experience {
    pub title: String,
    pub company: String,
    pub location: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub current: bool,
    pub description: Option<String>,
}

/// Compétence professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Skill {
    pub name: String,
}

/// Education académique ou professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Education {
    pub degree: String,
    pub school: String,
    pub location: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub description: Option<String>,
}

/// Language parlée et niveau associé.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Language {
    pub name: String,
    pub level: String,
}

/// Project personnel ou professionnel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Project {
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub technologies: Option<String>,
}

/// Certification obtenue.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Certification {
    pub name: String,
    pub issuer: Option<String>,
    pub date: Option<String>,
    pub url: Option<String>,
}

/// Profile complet persisté dans la ligne singleton `profil`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Profile {
    pub identity: Identity,
    pub experiences: Vec<Experience>,
    pub skills: Vec<Skill>,
    pub education: Vec<Education>,
    pub languages: Vec<Language>,
    pub projects: Vec<Project>,
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
