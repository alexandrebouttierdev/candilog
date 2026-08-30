//! Taille de l'entreprise.

use serde::{Deserialize, Serialize};

/// Taille de l'entreprise, dimension **distincte** de sa nature.
///
/// Une ESN peut être une PME et un éditeur SaaS une grande entreprise : mélanger les deux
/// dans un enum unique rendrait la moitié des combinaisons inexprimables.
///
/// `Unknown` plutôt que `Option` : la colonne est `NOT NULL DEFAULT 'UNKNOWN'`, ce qui ne
/// laisse qu'une seule représentation du « non renseigné » à filtrer et à afficher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "companies.ts")]
pub enum CompanySize {
    /// Micro-entreprise.
    #[serde(rename = "MICRO")]
    Micro,
    /// Très petite entreprise.
    #[serde(rename = "TPE")]
    Tpe,
    /// Petite ou moyenne entreprise.
    #[serde(rename = "PME")]
    Pme,
    /// Entreprise de taille intermédiaire.
    #[serde(rename = "ETI")]
    Eti,
    /// Grande entreprise ou grand groupe.
    #[serde(rename = "LARGE")]
    Large,
    /// Taille non renseignée (défaut).
    #[default]
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl std::fmt::Display for CompanySize {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Micro => "Micro-entreprise",
            Self::Tpe => "TPE",
            Self::Pme => "PME",
            Self::Eti => "ETI",
            Self::Large => "Grande entreprise / Grand groupe",
            Self::Unknown => "Non renseignée",
        })
    }
}
