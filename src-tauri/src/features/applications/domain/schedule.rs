//! Durée hebdomadaire de travail.

use serde::{Deserialize, Serialize};

/// Nombre d'heures au-delà duquel une durée hebdomadaire n'est plus réaliste.
///
/// Une semaine compte 168 heures : au-delà, la saisie est nécessairement une erreur
/// d'unité — un volume mensuel ou annuel entré dans un champ hebdomadaire.
pub const MAX_WEEKLY_HOURS: f64 = 168.0;

/// Régime horaire hebdomadaire du poste.
///
/// Distinct de [`weekly_hours`](crate::features::applications::domain::Application::weekly_hours),
/// qui en donne le volume : un temps partiel peut valoir 24 h comme 17,5 h, et un temps
/// plein n'a pas la même durée d'une convention à l'autre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "applications.ts")]
pub enum WeeklyWorkSchedule {
    /// Temps plein.
    #[serde(rename = "FULL_TIME")]
    FullTime,
    /// Temps partiel.
    #[serde(rename = "PART_TIME")]
    PartTime,
    /// Non renseignée (défaut).
    #[default]
    #[serde(rename = "UNSPECIFIED")]
    Unspecified,
}

impl WeeklyWorkSchedule {
    /// Traduit le code France Travail d'une durée hebdomadaire.
    ///
    /// Point de conversion **unique** : le jour où un import France Travail existera, il
    /// appellera cette fonction plutôt que de semer des `if value == "1"` dans le code, où
    /// chaque copie deviendrait un endroit de plus à corriger.
    ///
    /// Tout code hors `0` / `1` / `2` est traité comme non renseigné : un flux externe n'est
    /// pas une source de confiance, et refuser l'import entier pour une valeur inattendue
    /// coûterait plus que d'ignorer un champ.
    #[must_use]
    pub fn from_france_travail_code(code: &str) -> Self {
        match code.trim() {
            "1" => Self::FullTime,
            "2" => Self::PartTime,
            _ => Self::Unspecified,
        }
    }
}

impl std::fmt::Display for WeeklyWorkSchedule {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::FullTime => "Temps plein",
            Self::PartTime => "Temps partiel",
            Self::Unspecified => "Non renseignée",
        })
    }
}

#[cfg(test)]
#[path = "tests/schedule/mod.rs"]
mod tests;
