//! Canal par lequel l'offre a été trouvée — « Trouvée via » dans le formulaire.

use crate::features::applications::domain::application_type::ApplicationType;
use serde::{Deserialize, Serialize};

/// Canal d'une candidature.
///
/// Il détermine la nature de la démarche ([`ApplicationType`]) : seule une candidature
/// spontanée est [`ApplicationType::Unsolicited`]. Il fixe aussi le régime du lien de
/// l'offre — requis pour une offre publiée, facultatif pour le site de l'entreprise ou le
/// réseau, interdit pour une démarche spontanée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export, export_to = "applications.ts")]
pub enum ApplicationChannel {
    /// Offre publiée sur un site d'emploi (défaut).
    #[default]
    Offer,
    /// Offre publiée sur le site de l'entreprise.
    CompanySite,
    /// Réseau : cooptation, recommandation, contact.
    Network,
    /// Démarche spontanée, sans offre.
    Spontaneous,
}

impl ApplicationChannel {
    /// Nature de la démarche qui découle du canal.
    #[must_use]
    pub const fn application_type(self) -> ApplicationType {
        match self {
            Self::Spontaneous => ApplicationType::Unsolicited,
            Self::Offer | Self::CompanySite | Self::Network => ApplicationType::JobOffer,
        }
    }
}

impl std::fmt::Display for ApplicationChannel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Offer => "Offre",
            Self::CompanySite => "Site",
            Self::Network => "Réseau",
            Self::Spontaneous => "Spontanée",
        })
    }
}
