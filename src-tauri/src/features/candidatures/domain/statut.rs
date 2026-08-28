//! Statut et type de contrat d'une candidature.

use serde::{Deserialize, Serialize};

/// Étape de la candidature dans le pipeline.
///
/// Les quatre valeurs sont contraintes en base par un `CHECK` (migration 005) : y ajouter
/// une variante demande une migration, ce qui est voulu — le Kanban a une colonne par
/// statut, et un statut inconnu casserait la répartition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export, export_to = "candidatures.ts")]
pub enum StatutCandidature {
    /// En attente de réponse (défaut).
    EnAttente,
    /// Relancée après envoi.
    Relancee,
    /// En phase d'entretien.
    Entretien,
    /// Refusée.
    Refus,
}

impl std::fmt::Display for StatutCandidature {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::EnAttente => "En attente",
            Self::Relancee => "Relancée",
            Self::Entretien => "Entretien",
            Self::Refus => "Refusée",
        })
    }
}

/// Type de contrat visé.
///
/// Les valeurs sérialisées reprennent la casse exacte stockée en base et contrainte par le
/// `CHECK` de la migration 005 : les renommer romprait la lecture des données existantes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "candidatures.ts")]
pub enum TypeContrat {
    /// Contrat à durée indéterminée.
    #[serde(rename = "CDI")]
    Cdi,
    /// Contrat à durée déterminée.
    #[serde(rename = "CDD")]
    Cdd,
    /// Mission freelance.
    #[serde(rename = "Freelance")]
    Freelance,
    /// Stage.
    #[serde(rename = "Stage")]
    Stage,
    /// Alternance.
    #[serde(rename = "Alternance")]
    Alternance,
    /// Intérim.
    #[serde(rename = "Interim")]
    Interim,
    /// Autre type de contrat.
    #[serde(rename = "Autre")]
    Autre,
}

impl std::fmt::Display for TypeContrat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Cdi => "CDI",
            Self::Cdd => "CDD",
            Self::Freelance => "Freelance",
            Self::Stage => "Stage",
            Self::Alternance => "Alternance",
            Self::Interim => "Intérim",
            Self::Autre => "Autre",
        })
    }
}
