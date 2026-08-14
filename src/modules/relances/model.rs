//! Types du domaine des relances.

use serde::{Deserialize, Serialize};

/// Relance effectuée sur une candidature, telle que persistée.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relance {
    /// Identifiant de la relance.
    pub id: uuid::Uuid,
    /// Identifiant de la candidature relancée (FK `candidatures`, requis).
    pub candidature_id: uuid::Uuid,
    /// Date de la relance (ISO 8601).
    pub date_relance: String,
    /// Canal de relance (texte libre côté base : `Email`/`Téléphone`/`LinkedIn`/`Autre`).
    #[serde(rename = "type")]
    pub type_relance: String,
    /// Notes libres, si renseignées.
    pub notes: Option<String>,
    /// Date de création (ISO 8601).
    pub created_at: String,
}

/// Champs de création/édition d'une relance (`candidature_id` et `date_relance` requis).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NouvelleRelance {
    /// Identifiant de la candidature relancée (requis).
    pub candidature_id: uuid::Uuid,
    /// Date de la relance (ISO 8601).
    pub date_relance: String,
    /// Canal de relance.
    #[serde(rename = "type")]
    pub type_relance: String,
    /// Notes libres.
    pub notes: Option<String>,
}
