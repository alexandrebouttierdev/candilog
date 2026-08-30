//! Nature de la démarche : réponse à une offre ou candidature spontanée.

use serde::{Deserialize, Serialize};

/// Nature de la candidature.
///
/// Détermine le régime du lien de l'offre : obligatoire pour une réponse à une offre,
/// interdit pour une démarche spontanée — un lien conservé après bascule pointerait vers
/// une annonce qui n'a plus rien à voir avec la candidature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "applications.ts")]
pub enum ApplicationType {
    /// Réponse à une offre publiée (défaut).
    #[default]
    #[serde(rename = "OFFRE")]
    JobOffer,
    /// Démarche spontanée, sans offre associée.
    #[serde(rename = "SPONTANEE")]
    Unsolicited,
}

impl std::fmt::Display for ApplicationType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::JobOffer => "Offre d'emploi",
            Self::Unsolicited => "Candidature spontanée",
        })
    }
}
