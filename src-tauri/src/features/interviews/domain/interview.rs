//! Entité, type et champs éditables d'un entretien.

use serde::{Deserialize, Serialize};

/// Format de l'entretien.
///
/// Les valeurs sérialisées reprennent la casse exacte contrainte en base par le `CHECK` de
/// `init_schema`, accents compris : les renommer romprait la lecture des lignes existantes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "interviews.ts")]
pub enum InterviewType {
    /// Interview en présentiel (défaut).
    #[default]
    #[serde(rename = "Présentiel")]
    OnSite,
    /// Interview en visioconférence.
    #[serde(rename = "Visio")]
    Video,
    /// Interview téléphonique.
    #[serde(rename = "Téléphonique")]
    Phone,
    /// Interview technique.
    #[serde(rename = "Technique")]
    Technical,
    /// Interview avec les ressources humaines.
    #[serde(rename = "RH")]
    Hr,
    /// Autre format.
    #[serde(rename = "Autre")]
    Other,
}

impl std::fmt::Display for InterviewType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::OnSite => "Présentiel",
            Self::Video => "Visio",
            Self::Phone => "Téléphonique",
            Self::Technical => "Technique",
            Self::Hr => "RH",
            Self::Other => "Autre",
        })
    }
}

/// Analysis `IA` du compte rendu d'un entretien.
///
/// Persistée en `TEXT` `JSON` sur l'entretien. Définie ici plutôt que dans la feature `ia` :
/// c'est un champ de l'entretien, et la faire vivre ailleurs obligerait `entretiens` à
/// dépendre de l'IA pour lire ses propres lignes.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "interviews.ts")]
pub struct InterviewAnalysis {
    /// Résumé synthétique de l'entretien.
    pub resume: String,
    /// Points forts relevés dans le compte rendu.
    pub strengths: Vec<String>,
    /// Points faibles relevés dans le compte rendu.
    pub weaknesses: Vec<String>,
    /// Suggestions pour les prochains entretiens.
    pub suggestions: Vec<String>,
}

/// Interview rattaché à une candidature, tel que persisté.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "interviews.ts")]
pub struct Interview {
    /// Id de l'entretien.
    pub id: uuid::Uuid,
    /// Id de la candidature concernée.
    pub application_id: uuid::Uuid,
    /// Intitulé du poste, aplati depuis la jointure — ce que le calendrier affiche.
    pub application_job_title: Option<String>,
    /// Name de l'entreprise, aplati depuis la jointure.
    pub company_name: Option<String>,
    /// Id du contact lié, s'il existe.
    pub contact_id: Option<uuid::Uuid>,
    /// Name complet de l'interlocuteur, aplati depuis la jointure.
    pub contact_name: Option<String>,
    /// Date et heure de l'entretien (ISO 8601).
    pub interview_date: String,
    /// Format de l'entretien.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub type_interview: InterviewType,
    /// Location (présentiel) ou lien (visio).
    pub location: Option<String>,
    /// Notes de préparation.
    pub notes: Option<String>,
    /// Report rendu rédigé après l'entretien.
    pub minutes: Option<String>,
    /// Analysis `IA` du compte rendu, si elle a été produite.
    #[serde(default)]
    pub analysis_ai: Option<InterviewAnalysis>,
    /// Date de création (ISO 8601).
    pub created_at: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
}

/// Champs éditables d'un entretien, en création comme en modification.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "interviews.ts")]
pub struct NewInterview {
    /// Id de la candidature concernée (requis).
    pub application_id: uuid::Uuid,
    /// Id du contact lié.
    pub contact_id: Option<uuid::Uuid>,
    /// Date et heure de l'entretien (ISO 8601).
    pub interview_date: String,
    /// Format de l'entretien.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub type_interview: InterviewType,
    /// Location ou lien.
    pub location: Option<String>,
    /// Notes de préparation.
    pub notes: Option<String>,
    /// Report rendu.
    pub minutes: Option<String>,
}
