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

    /// Activité de la relation, calculée en base à chaque lecture.
    pub activity: CompanyActivity,
}

/// Activité d'une entreprise dans le suivi : ce qui la range dans « En cours »,
/// « Repérées » ou « Clôturées » (écran Relations).
///
/// Calculée par sous-requêtes et jamais stockée : une colonne de compteur dériverait à la
/// première candidature supprimée hors du chemin prévu.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "companies.ts")]
pub struct CompanyActivity {
    /// Candidatures non refusées.
    pub open_applications: u32,
    /// Toutes les candidatures envoyées à l'entreprise.
    pub applications: u32,
    /// Contacts rattachés à l'entreprise.
    pub contacts: u32,
    /// Numéro de référence de la candidature la plus récente.
    #[ts(type = "number | null")]
    pub last_reference_number: Option<i64>,
    /// Date d'envoi de la candidature la plus récente (`AAAA-MM-JJ`).
    pub last_sent_date: Option<String>,
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
