//! Types du domaine de la configuration applicative.

use crate::shared::llm::LlmConfig;
use serde::{Deserialize, Serialize};

/// Préférence de thème de l'interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePref {
    /// Thème clair.
    Light,
    /// Thème sombre.
    Dark,
    /// Suit le réglage système.
    System,
}

impl std::fmt::Display for ThemePref {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Light => "Clair",
            Self::Dark => "Sombre",
            Self::System => "Système",
        })
    }
}

/// Paramètres complets de l'application (persistés dans `app_settings`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Configuration LLM.
    pub llm: LlmConfig,
    /// Préférence de thème.
    pub theme: ThemePref,
    /// Langue de l'interface (code ISO, ex : "fr").
    pub langue: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            theme: ThemePref::System,
            langue: "fr".into(),
        }
    }
}

#[cfg(test)]
#[path = "tests/model/mod.rs"]
mod tests;
