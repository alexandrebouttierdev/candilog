//! Status et type de contrat d'une candidature.

use serde::{Deserialize, Serialize};

/// Étape de la candidature dans le pipeline.
///
/// Les quatre valeurs sont contraintes en base par un `CHECK` (`init_schema`) : y ajouter
/// une variante demande une migration, ce qui est voulu — le Kanban a une colonne par
/// statut, et un statut inconnu casserait la répartition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "applications.ts")]
pub enum ApplicationStatus {
    /// En attente de réponse (défaut).
    #[serde(rename = "EN_ATTENTE")]
    Pending,
    /// Relancée après envoi.
    #[serde(rename = "RELANCEE")]
    FollowedUp,
    /// En phase d'entretien.
    #[serde(rename = "ENTRETIEN")]
    Interview,
    /// Candidature refusée.
    #[serde(rename = "REFUS")]
    Rejected,
}

impl std::fmt::Display for ApplicationStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Pending => "En attente",
            Self::FollowedUp => "Relancée",
            Self::Interview => "Entretien",
            Self::Rejected => "Refusée",
        })
    }
}

/// Type de contrat visé.
///
/// Les valeurs sérialisées reprennent la casse exacte stockée en base et contrainte par le
/// `CHECK` de `init_schema` : les renommer romprait la lecture des données existantes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "applications.ts")]
pub enum ContractType {
    /// Contract à durée indéterminée.
    #[serde(rename = "CDI")]
    Cdi,
    /// Contract à durée déterminée.
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
    Other,
}

impl std::fmt::Display for ContractType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Cdi => "CDI",
            Self::Cdd => "CDD",
            Self::Freelance => "Freelance",
            Self::Stage => "Stage",
            Self::Alternance => "Alternance",
            Self::Interim => "Intérim",
            Self::Other => "Autre",
        })
    }
}
