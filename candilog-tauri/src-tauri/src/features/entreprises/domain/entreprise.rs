//! Entité et champs éditables d'une entreprise.

use serde::{Deserialize, Serialize};

/// Entreprise telle que persistée.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "entreprises.ts")]
pub struct Entreprise {
    /// Identifiant de l'entreprise.
    pub id: uuid::Uuid,
    /// Nom de l'entreprise.
    pub nom: String,
    /// Identifiant du secteur d'activité lié (référentiel `secteurs_activite`).
    pub secteur_id: Option<uuid::Uuid>,
    /// Libellé du secteur, dénormalisé depuis le référentiel.
    ///
    /// Conservé en plus de `secteur_id` parce que l'ancienne base porte des secteurs saisis
    /// librement, sans ligne de référentiel correspondante ; la migration 008 les rattache
    /// mais le libellé reste la seule valeur sûre pour l'affichage et la recherche.
    pub secteur: Option<String>,
    /// Type d'entreprise (colonne `type`), s'il est renseigné.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub type_: Option<String>,
    /// Site web, s'il est renseigné.
    pub site_web: Option<String>,
    /// Ville, si elle est renseignée.
    pub ville: Option<String>,
    /// Adresse postale, si elle est renseignée.
    pub adresse: Option<String>,
    /// Notes libres, si renseignées.
    pub notes: Option<String>,
    /// Date de création (ISO 8601).
    pub created_at: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
}

/// Champs de création et d'édition d'une entreprise : seul le nom est requis.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "entreprises.ts")]
pub struct NouvelleEntreprise {
    /// Nom de l'entreprise (requis).
    pub nom: String,
    /// Secteur choisi dans le référentiel `secteurs_activite`.
    pub secteur_id: Option<uuid::Uuid>,
    /// Libellé du secteur, dénormalisé depuis le référentiel.
    pub secteur: Option<String>,
    /// Type d'entreprise (colonne `type`).
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub type_: Option<String>,
    /// Site web.
    pub site_web: Option<String>,
    /// Ville.
    pub ville: Option<String>,
    /// Adresse postale.
    pub adresse: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
}

/// Édition d'une entreprise : remplacement complet, identique à la création.
pub type MajEntreprise = NouvelleEntreprise;
