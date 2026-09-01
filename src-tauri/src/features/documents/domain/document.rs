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
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct NewResume {
    pub name: String,
    #[ts(type = "unknown")]
    pub content: serde_json::Value,
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
#[derive(Debug, Clone, Deserialize, TS)]
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
}

/// Content d'une lettre à exporter en PDF (enregistrée ou encore à l'écran).
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
