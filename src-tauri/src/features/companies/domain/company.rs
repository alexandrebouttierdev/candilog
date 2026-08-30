//! Entité et champs éditables d'une entreprise.

use crate::features::companies::domain::company_size::CompanySize;
use serde::{Deserialize, Serialize};

/// Entreprise telle que persistée, libellés des référentiels aplatis depuis les jointures.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "companies.ts")]
pub struct Company {
    /// Id de l'entreprise.
    pub id: uuid::Uuid,
    /// Nom de l'entreprise.
    pub name: String,

    /// Secteur **d'activité de l'entreprise** (référentiel `sectors`).
    ///
    /// Ne décrit jamais le métier recherché : celui-ci relève du domaine professionnel de
    /// la candidature.
    pub sector_id: Option<uuid::Uuid>,
    /// Libellé du secteur, aplati depuis la jointure sur `sectors`.
    ///
    /// Résolu par `JOIN` et non stocké : une seconde colonne de libellé donnerait deux
    /// sources de vérité, que rien ne garderait d'accord.
    pub sector_name: Option<String>,

    /// Nature de l'organisation (référentiel `company_types`).
    pub company_type_id: Option<String>,
    /// Libellé du type d'entreprise, aplati depuis la jointure sur `company_types`.
    pub company_type_name: Option<String>,

    /// Taille de l'entreprise, dimension distincte de sa nature.
    pub company_size: CompanySize,

    /// Site web, s'il est renseigné.
    pub website: Option<String>,
    /// Ville du siège ou de l'implantation principale.
    pub city: Option<String>,
    /// Adresse du siège ou de l'implantation principale.
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
    /// Nom de l'entreprise (requis).
    pub name: String,
    /// Secteur d'activité choisi dans le référentiel `sectors`.
    pub sector_id: Option<uuid::Uuid>,
    /// Nature de l'organisation, choisie dans le référentiel `company_types`.
    pub company_type_id: Option<String>,
    /// Taille de l'entreprise.
    #[serde(default)]
    pub company_size: CompanySize,
    /// Site web.
    pub website: Option<String>,
    /// Ville du siège ou de l'implantation principale.
    pub city: Option<String>,
    /// Adresse du siège ou de l'implantation principale.
    pub address: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
}

/// Édition d'une entreprise : remplacement complet, identique à la création.
pub type CompanyUpdate = NewCompany;
