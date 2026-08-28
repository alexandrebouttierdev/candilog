//! Période d'observation des analyses.

use serde::{Deserialize, Serialize};

/// Fenêtre temporelle sur laquelle les indicateurs sont calculés.
///
/// Jeu fermé plutôt qu'un nombre de jours libre : la valeur sert à borner des requêtes, et
/// les maquettes n'offrent que ces trois choix. Un entier venu de l'IPC obligerait à le
/// valider, sans rien apporter à l'utilisateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "analyses.ts")]
pub enum Periode {
    /// Trente derniers jours (défaut des maquettes).
    #[default]
    TrenteJours,
    /// Quatre-vingt-dix derniers jours.
    QuatreVingtDixJours,
    /// Tout l'historique.
    Tout,
}

impl Periode {
    /// Date `AAAA-MM-JJ` à partir de laquelle observer, ou `None` pour tout l'historique.
    ///
    /// La borne est calculée à partir de la date **locale** : un utilisateur qui consulte
    /// ses analyses à 1 h du matin attend la fenêtre de sa journée, pas de celle d'UTC.
    #[must_use]
    pub fn depuis(self, aujourdhui: chrono::NaiveDate) -> Option<String> {
        let jours = match self {
            Self::TrenteJours => 30,
            Self::QuatreVingtDixJours => 90,
            Self::Tout => return None,
        };
        Some(
            (aujourdhui - chrono::Duration::days(jours))
                .format("%Y-%m-%d")
                .to_string(),
        )
    }

    /// Nombre de semaines représentées par le graphique d'activité.
    ///
    /// La vue « Tout » reste bornée à un an : au-delà, des barres hebdomadaires de quelques
    /// pixels ne seraient plus lisibles. Les indicateurs, eux, portent bien sur tout
    /// l'historique ; seule la visualisation conserve une fenêtre utile.
    #[must_use]
    pub const fn semaines(self) -> u32 {
        match self {
            Self::TrenteJours => 5,
            Self::QuatreVingtDixJours => 13,
            Self::Tout => 52,
        }
    }
}
