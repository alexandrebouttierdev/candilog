//! Entité et champs éditables d'une relance.

use serde::{Deserialize, Serialize};

/// Relance effectuée sur une candidature, telle que persistée.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "relances.ts")]
pub struct Relance {
    /// Identifiant de la relance.
    pub id: uuid::Uuid,
    /// Identifiant de la candidature relancée.
    pub candidature_id: uuid::Uuid,
    /// Intitulé du poste, aplati depuis la jointure — ce que le calendrier affiche.
    pub candidature_poste: Option<String>,
    /// Nom de l'entreprise, aplati depuis la jointure.
    pub entreprise_nom: Option<String>,
    /// Date de la relance (`AAAA-MM-JJ`).
    pub date_relance: String,
    /// Canal de relance.
    ///
    /// Texte libre en base, sans contrainte `CHECK` : l'interface propose quatre canaux
    /// courants, mais les lignes héritées peuvent en porter d'autres.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub type_relance: String,
    /// Notes libres.
    pub notes: Option<String>,
    /// Date de création (ISO 8601).
    pub created_at: String,
}

/// Champs éditables d'une relance, en création comme en modification.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "relances.ts")]
pub struct NouvelleRelance {
    /// Identifiant de la candidature relancée (requis).
    pub candidature_id: uuid::Uuid,
    /// Date de la relance (`AAAA-MM-JJ`).
    pub date_relance: String,
    /// Canal de relance.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub type_relance: String,
    /// Notes libres.
    pub notes: Option<String>,
}
