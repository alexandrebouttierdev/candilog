//! Entité et champs éditables d'une entreprise.

use serde::{Deserialize, Serialize};

/// Company telle que persistée.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "companies.ts")]
pub struct Company {
    /// Id de l'entreprise.
    pub id: uuid::Uuid,
    /// Name de l'entreprise.
    pub name: String,
    /// Id du secteur d'activité lié (référentiel `sectors_activity`).
    pub sector_id: Option<uuid::Uuid>,
    /// Libellé du secteur, dénormalisé depuis le référentiel.
    ///
    /// Conservé en plus de `sector_id` parce que l'ancienne base porte des secteurs saisis
    /// librement, sans ligne de référentiel correspondante ; la migration 008 les rattache
    /// mais le libellé reste la seule valeur sûre pour l'affichage et la recherche.
    pub sector: Option<String>,
    /// Type d'entreprise (colonne `type`), s'il est renseigné.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub type_: Option<String>,
    /// Site web, s'il est renseigné.
    pub website: Option<String>,
    /// City, si elle est renseignée.
    pub city: Option<String>,
    /// Address postale, si elle est renseignée.
    pub address: Option<String>,
    /// Notes libres, si renseignées.
    pub notes: Option<String>,
    /// Date de création (ISO 8601).
    pub created_at: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
}

/// Champs de création et d'édition d'une entreprise : seul le nom est requis.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "companies.ts")]
pub struct NewCompany {
    /// Name de l'entreprise (requis).
    pub name: String,
    /// Sector choisi dans le référentiel `sectors_activity`.
    pub sector_id: Option<uuid::Uuid>,
    /// Libellé du secteur, dénormalisé depuis le référentiel.
    pub sector: Option<String>,
    /// Type d'entreprise (colonne `type`).
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub type_: Option<String>,
    /// Site web.
    pub website: Option<String>,
    /// City.
    pub city: Option<String>,
    /// Address postale.
    pub address: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
}

/// Édition d'une entreprise : remplacement complet, identique à la création.
pub type CompanyUpdate = NewCompany;
