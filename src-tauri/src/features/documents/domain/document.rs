//! Types persistés dans les bibliothèques Documents.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Résumé léger d'une version de CV.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeSummary {
    pub id: Uuid,
    pub name: String,
    pub created_at: String,
    /// Score ATS enregistré avec la version (0–100), s'il a été calculé.
    ///
    /// Lu dans le JSON à la volée : le contenu reste la seule source, et une ancienne
    /// génération (`profile_score`) comme un éditeur v1 (`score`) le portent.
    pub ats_score: Option<u32>,
    /// Intitulé de l'offre ciblée, s'il y en a une.
    pub target_title: Option<String>,
}

/// Version complète de CV ; son contenu structuré reste extensible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeVersion {
    pub id: Uuid,
    pub name: String,
    #[ts(type = "unknown")]
    pub content: serde_json::Value,
    pub created_at: String,
}

/// Entrée d'enregistrement d'un CV.
#[derive(Debug, Clone, Default, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct NewResume {
    pub name: String,
    #[ts(type = "unknown")]
    pub content: serde_json::Value,
    /// Document que cet enregistrement révise (l'une quelconque de ses versions) : il en
    /// devient la version suivante, courante. Absent : un nouveau document, en v1.
    #[serde(default)]
    #[ts(optional)]
    pub revises: Option<Uuid>,
    /// Ce qui distingue cette version (« Première génération », « Modifiée dans l'éditeur »).
    #[serde(default)]
    #[ts(optional)]
    pub version_note: Option<String>,
}

/// CoverLetter enregistrée dans la bibliothèque locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct CoverLetter {
    pub id: Uuid,
    pub name: String,
    pub company: Option<String>,
    pub job_title: Option<String>,
    #[serde(default)]
    pub recipient: Option<String>,
    #[serde(default)]
    pub recipient_address: Option<String>,
    #[serde(default)]
    pub job_reference: Option<String>,
    pub tone: String,
    pub length: String,
    pub content: String,
    pub created_at: String,
}

/// Entrée d'enregistrement d'une lettre générée ou remaniée.
#[derive(Debug, Clone, Default, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct NewCoverLetter {
    pub name: String,
    pub company: Option<String>,
    pub job_title: Option<String>,
    #[serde(default)]
    pub recipient: Option<String>,
    #[serde(default)]
    pub recipient_address: Option<String>,
    #[serde(default)]
    pub job_reference: Option<String>,
    pub tone: String,
    pub length: String,
    pub content: String,
    /// Document que cet enregistrement révise (l'une quelconque de ses versions) : il en
    /// devient la version suivante, courante. Absent : un nouveau document, en v1.
    #[serde(default)]
    #[ts(optional)]
    pub revises: Option<Uuid>,
    /// Ce qui distingue cette version (« Première génération », « Modifiée dans l'éditeur »).
    #[serde(default)]
    #[ts(optional)]
    pub version_note: Option<String>,
}

/// Version d'un document, telle que la liste l'inspecteur (`screens/07`, « Versions »).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct DocumentVersion {
    pub id: Uuid,
    /// Rang dans le document : 1 pour v1.
    pub version_number: u32,
    pub note: Option<String>,
    pub created_at: String,
    /// Version que la bibliothèque affiche.
    pub is_current: bool,
}

/// Contenu d'une lettre à exporter en PDF (enregistrée ou encore à l'écran).
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct CoverLetterExport {
    pub name: String,
    pub company: Option<String>,
    pub job_title: Option<String>,
    #[serde(default)]
    pub recipient: Option<String>,
    #[serde(default)]
    pub recipient_address: Option<String>,
    #[serde(default)]
    pub job_reference: Option<String>,
    pub content: String,
}
