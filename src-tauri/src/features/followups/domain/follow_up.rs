//! Entité et champs éditables d'une relance.

use serde::{Deserialize, Serialize};

/// FollowUp effectuée sur une candidature, telle que persistée.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "followups.ts")]
pub struct FollowUp {
    /// Id de la relance.
    pub id: uuid::Uuid,
    /// Id de la candidature relancée.
    pub application_id: uuid::Uuid,
    /// Intitulé du poste, aplati depuis la jointure — ce que le calendrier affiche.
    pub application_job_title: Option<String>,
    /// Name de l'entreprise, aplati depuis la jointure.
    pub company_name: Option<String>,
    /// Date de la relance (`AAAA-MM-JJ`).
    pub follow_up_date: String,
    /// Channel de relance.
    ///
    /// Text libre en base, sans contrainte `CHECK` : l'interface propose quatre canaux
    /// courants, mais les lignes héritées peuvent en porter d'autres.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub channel: String,
    /// Notes libres.
    pub notes: Option<String>,
    /// Date de création (ISO 8601).
    pub created_at: String,
}

/// Champs éditables d'une relance, en création comme en modification.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "followups.ts")]
pub struct NewFollowUp {
    /// Id de la candidature relancée (requis).
    pub application_id: uuid::Uuid,
    /// Date de la relance (`AAAA-MM-JJ`).
    pub follow_up_date: String,
    /// Channel de relance.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub channel: String,
    /// Notes libres.
    pub notes: Option<String>,
}
