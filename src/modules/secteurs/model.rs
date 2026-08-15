//! Types du référentiel des secteurs d'activité.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Secteur d'activité du référentiel `secteurs_activite`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecteurActivite {
    /// Identifiant du secteur.
    pub id: uuid::Uuid,
    /// Libellé affiché dans les sélecteurs.
    pub nom: String,
}

impl fmt::Display for SecteurActivite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.nom)
    }
}
