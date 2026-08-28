//! Types persistés dans les bibliothèques Documents.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Résumé léger d'une version de CV.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "documents.ts")]
pub struct CvResume {
    pub id: Uuid,
    pub nom: String,
    pub created_at: String,
}

/// Version complète de CV ; son contenu structuré reste extensible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "documents.ts")]
pub struct CvVersion {
    pub id: Uuid,
    pub nom: String,
    #[ts(type = "unknown")]
    pub contenu: serde_json::Value,
    pub created_at: String,
}

/// Entrée d'enregistrement d'un CV.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "documents.ts")]
pub struct NouveauCv {
    pub nom: String,
    #[ts(type = "unknown")]
    pub contenu: serde_json::Value,
}

/// Lettre enregistrée dans la bibliothèque locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "documents.ts")]
pub struct Lettre {
    pub id: Uuid,
    pub nom: String,
    pub entreprise: Option<String>,
    pub poste: Option<String>,
    pub ton: String,
    pub longueur: String,
    pub contenu: String,
    pub created_at: String,
}

/// Entrée d'enregistrement d'une lettre générée ou remaniée.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "documents.ts")]
pub struct NouvelleLettre {
    pub nom: String,
    pub entreprise: Option<String>,
    pub poste: Option<String>,
    pub ton: String,
    pub longueur: String,
    pub contenu: String,
}

/// Contenu d'une lettre à exporter en PDF (enregistrée ou encore à l'écran).
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "documents.ts")]
pub struct ExportLettre {
    pub nom: String,
    pub entreprise: Option<String>,
    pub poste: Option<String>,
    pub contenu: String,
}
