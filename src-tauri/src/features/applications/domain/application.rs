//! Entité et champs éditables d'une candidature.

use crate::features::applications::domain::status::{ApplicationStatus, ContractType};
use serde::{Deserialize, Serialize};

/// Application telle que persistée, nom d'entreprise aplati depuis la jointure.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "applications.ts")]
pub struct Application {
    /// Id de la candidature.
    pub id: uuid::Uuid,
    /// Intitulé du poste visé.
    pub job_title: String,
    /// Id de l'entreprise liée.
    pub company_id: uuid::Uuid,
    /// Name de l'entreprise liée, aplati depuis la jointure ; `None` si non résolu.
    pub company_name: Option<String>,
    /// City de l'entreprise liée, aplatie depuis la jointure.
    ///
    /// Affichée dans la colonne « Company » de la vue List et sur les cartes du Kanban :
    /// sans elle, chaque ligne devrait relire le répertoire des entreprises.
    pub company_city: Option<String>,
    /// Id du contact lié, s'il existe.
    pub contact_id: Option<uuid::Uuid>,
    /// Type de contrat visé.
    pub contract_type: ContractType,
    /// Status courant dans le pipeline.
    pub status: ApplicationStatus,
    /// Date d'envoi, au format `AAAA-MM-JJ`.
    ///
    /// Les lignes reprises de l'ancienne base peuvent porter un horodatage ISO 8601
    /// complet : le tri et l'affichage restent corrects, mais le format n'est pas homogène.
    pub sent_date: String,
    /// Url vers l'offre d'origine, s'il existe.
    pub job_url: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
    /// Date de création (ISO 8601).
    pub created_at: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
}

/// Champs éditables d'une candidature, en création comme en modification.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "applications.ts")]
pub struct NewApplication {
    /// Intitulé du poste visé.
    pub job_title: String,
    /// Id de l'entreprise liée (requis).
    pub company_id: uuid::Uuid,
    /// Type de contrat visé.
    pub contract_type: ContractType,
    /// Status initial ou cible.
    pub status: ApplicationStatus,
    /// Date d'envoi choisie par l'utilisateur, au format `AAAA-MM-JJ`.
    pub sent_date: String,
    /// Url vers l'offre, s'il existe.
    pub job_url: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
}
