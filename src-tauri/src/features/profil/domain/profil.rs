//! Entités du profil exposées à React.

use serde::{Deserialize, Serialize};

/// Coordonnées et objectif professionnel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "profil.ts")]
pub struct Identite {
    pub prenom: String,
    pub nom: String,
    pub email: String,
    pub telephone: Option<String>,
    pub ville: Option<String>,
    /// Accroche courte, utilisée comme objectif ou titre de CV.
    pub titre: Option<String>,
    /// Présentation détaillée du parcours et de l'objectif.
    pub resume: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub site_web: Option<String>,
}

/// Expérience professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "profil.ts")]
pub struct Experience {
    pub intitule: String,
    pub entreprise: String,
    pub lieu: Option<String>,
    pub date_debut: String,
    pub date_fin: Option<String>,
    pub poste_actuel: bool,
    pub description: Option<String>,
}

/// Compétence professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "profil.ts")]
pub struct Competence {
    pub nom: String,
}

/// Formation académique ou professionnelle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "profil.ts")]
pub struct Formation {
    pub diplome: String,
    pub etablissement: String,
    pub lieu: Option<String>,
    pub date_debut: Option<String>,
    pub date_fin: Option<String>,
    pub description: Option<String>,
}

/// Langue parlée et niveau associé.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "profil.ts")]
pub struct Langue {
    pub nom: String,
    pub niveau: String,
}

/// Projet personnel ou professionnel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "profil.ts")]
pub struct Projet {
    pub nom: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub technologies: Option<String>,
}

/// Certification obtenue.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "profil.ts")]
pub struct Certification {
    pub nom: String,
    pub organisme: Option<String>,
    pub date: Option<String>,
    pub url: Option<String>,
}

/// Profil complet persisté dans la ligne singleton `profil`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "profil.ts")]
pub struct Profil {
    pub identite: Identite,
    pub experiences: Vec<Experience>,
    pub competences: Vec<Competence>,
    pub formations: Vec<Formation>,
    pub langues: Vec<Langue>,
    pub projets: Vec<Projet>,
    pub certifications: Vec<Certification>,
}

/// Charge utile de l'écran Profil.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "profil.ts")]
pub struct ProfilCharge {
    pub profil: Profil,
    /// Score de complétion entre 0 et 100.
    #[ts(type = "number")]
    pub completion: u8,
    /// Sections encore absentes, dans l'ordre utile à l'utilisateur.
    pub sections_incompletes: Vec<String>,
    /// Horodatage du dernier enregistrement, absent pour un profil neuf.
    pub updated_at: Option<String>,
}
