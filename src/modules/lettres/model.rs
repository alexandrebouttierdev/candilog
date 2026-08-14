//! Modèle persistant d'une lettre de motivation.

use serde::{Deserialize, Serialize};

/// Lettre enregistrée dans la bibliothèque locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LettreMotivation {
    pub id: uuid::Uuid,
    pub name: String,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub tone: String,
    pub length: String,
    pub content: String,
    pub created_at: String,
}
