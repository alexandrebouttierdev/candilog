//! Entité du référentiel des secteurs d'activité.

use serde::{Deserialize, Serialize};

/// Secteur d'activité du référentiel `secteurs_activite`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "secteurs.ts")]
pub struct SecteurActivite {
    /// Identifiant du secteur.
    pub id: uuid::Uuid,
    /// Libellé affiché dans les sélecteurs.
    pub nom: String,
}
