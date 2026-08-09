//! Types du domaine des versions de CV.

use serde::{Deserialize, Serialize};

/// Version de CV complète telle que persistée (contenu inclus).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvVersion {
    /// Identifiant de la version.
    pub id: uuid::Uuid,
    /// Nom lisible donné par l'utilisateur.
    pub name: String,
    /// Contenu applicatif opaque (forme définie par le frontend).
    pub content: serde_json::Value,
    /// Date de création (format ISO 8601).
    pub created_at: String,
}

/// Résumé d'une version pour l'affichage en liste (sans le contenu lourd).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvVersionSummary {
    /// Identifiant de la version.
    pub id: uuid::Uuid,
    /// Nom lisible donné par l'utilisateur.
    pub name: String,
    /// Date de création (ISO 8601).
    pub created_at: String,
}

#[cfg(test)]
#[path = "tests/model/mod.rs"]
mod tests;
