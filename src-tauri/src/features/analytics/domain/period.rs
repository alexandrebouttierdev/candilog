//! Période d'observation des analyses.

use serde::{Deserialize, Serialize};

/// Fenêtre temporelle sur laquelle les indicateurs sont calculés.
///
/// Jeu fermé plutôt qu'un nombre de jours libre : la valeur sert à borner des requêtes, et
/// les maquettes n'offrent que ces trois choix. Un entier venu de l'IPC obligerait à le
/// valider, sans rien apporter à l'utilisateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "analytics.ts")]
pub enum Period {
    /// Trente derniers jours (défaut des maquettes).
    #[default]
    TrenteDays,
    /// Quatre-vingt-dix derniers jours.
    QuatreVingtDixDays,
    /// Tout l'historique.
    Tout,
}

impl Period {
    /// Date `AAAA-MM-JJ` à partir de laquelle observer, ou `None` pour tout l'historique.
    ///
    /// La borne est calculée à partir de la date **locale** : un utilisateur qui consulte
    /// ses analyses à 1 h du matin attend la fenêtre de sa journée, pas de celle d'UTC.
    #[must_use]
    pub fn from(self, today: chrono::NaiveDate) -> Option<String> {
        let days = match self {
            Self::TrenteDays => 30,
            Self::QuatreVingtDixDays => 90,
            Self::Tout => return None,
        };
        Some(
            (today - chrono::Duration::days(days))
                .format("%Y-%m-%d")
                .to_string(),
        )
    }

    /// Count de semaines représentées par le graphique d'activité.
    ///
    /// La vue « Tout » reste bornée à un an : au-delà, des barres hebdomadaires de quelques
    /// pixels ne seraient plus lisibles. Les indicateurs, eux, portent bien sur tout
    /// l'historique ; seule la visualisation conserve une fenêtre utile.
    #[must_use]
    pub const fn semaines(self) -> u32 {
        match self {
            Self::TrenteDays => 5,
            Self::QuatreVingtDixDays => 13,
            Self::Tout => 52,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_period_json_suit_le_snake_case_de_serde() {
        assert_eq!(
            serde_json::to_string(&Period::TrenteDays).unwrap(),
            "\"trente_days\""
        );
        assert_eq!(
            serde_json::to_string(&Period::QuatreVingtDixDays).unwrap(),
            "\"quatre_vingt_dix_days\""
        );
        assert_eq!(serde_json::to_string(&Period::Tout).unwrap(), "\"tout\"");
        assert_eq!(
            serde_json::from_str::<Period>("\"trente_days\"").unwrap(),
            Period::TrenteDays
        );
    }
}
