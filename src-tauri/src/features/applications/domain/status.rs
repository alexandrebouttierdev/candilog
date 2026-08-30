//! Statut d'une candidature dans le pipeline.

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
