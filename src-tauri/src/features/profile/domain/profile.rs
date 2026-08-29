//! Entités du profil exposées à React.

use serde::{Deserialize, Serialize};

/// Coordonnées et objectif professionnel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Identity {
    #[serde(alias = "prenom")]
    pub first_name: String,
    #[serde(alias = "nom")]
    pub name: String,
    pub email: String,
    #[serde(alias = "telephone")]
    pub phone: Option<String>,
    #[serde(alias = "ville")]
    pub city: Option<String>,
    /// Accroche courte, utilisée comme objectif ou titre de CV.
    #[serde(alias = "titre")]
    pub title: Option<String>,
    /// Présentation détaillée du parcours et de l'objectif.
    pub resume: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    #[serde(alias = "siteWeb", alias = "site_web")]
    pub website: Option<String>,
}

/// Expérience professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Experience {
    #[serde(alias = "intitule")]
    pub title: String,
    #[serde(alias = "entreprise")]
    pub company: String,
    #[serde(alias = "lieu")]
    pub location: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    #[serde(alias = "posteActuel", alias = "poste_actuel")]
    pub current: bool,
    pub description: Option<String>,
}

/// Compétence professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Skill {
    #[serde(alias = "nom")]
    pub name: String,
}

/// Education académique ou professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Education {
    #[serde(alias = "diplome")]
    pub degree: String,
    #[serde(alias = "etablissement")]
    pub school: String,
    #[serde(alias = "lieu")]
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
    #[serde(alias = "nom")]
    pub name: String,
    #[serde(alias = "niveau")]
    pub level: String,
}

/// Project personnel ou professionnel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Project {
    #[serde(alias = "nom")]
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
    #[serde(alias = "nom")]
    pub name: String,
    #[serde(alias = "organisme")]
    pub issuer: Option<String>,
    pub date: Option<String>,
    pub url: Option<String>,
}

/// Profile complet persisté dans la ligne singleton `profil`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct Profile {
    #[serde(alias = "identite")]
    pub identity: Identity,
    pub experiences: Vec<Experience>,
    #[serde(alias = "competences")]
    pub skills: Vec<Skill>,
    #[serde(alias = "formations")]
    pub education: Vec<Education>,
    #[serde(alias = "langues")]
    pub languages: Vec<Language>,
    #[serde(alias = "projets")]
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
