//! Entité du référentiel des secteurs d'activité.

use serde::{Deserialize, Serialize};

/// Sector d'activité du référentiel `sectors_activity`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sectors.ts")]
pub struct ActivitySector {
    /// Id du secteur.
    pub id: uuid::Uuid,
    /// Libellé affiché dans les sélecteurs.
    pub name: String,
}
