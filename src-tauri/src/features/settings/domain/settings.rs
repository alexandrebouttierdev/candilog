//! Modèle persisté (JSON Iced) et DTO IPC camelCase.

use crate::features::ai::domain::{AnalysisMode, LlmConfig, ProviderKind};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Préférence de thème, identique à l'enum historique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub enum ThemePref {
    Light,
    Dark,
    #[default]
    System,
}

/// Paramètres complets tels qu'ils sont écrits dans `parametres.data`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub llm: LlmConfig,
    #[serde(default)]
    pub theme: ThemePref,
    #[serde(default = "language_fr")]
    pub language: String,
}

fn language_fr() -> String {
    "fr".into()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            theme: ThemePref::System,
            language: language_fr(),
        }
    }
}

/// Forme IPC destinée à React.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub struct Settings {
    pub llm: LlmForm,
    pub theme: ThemePref,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub struct LlmForm {
    pub provider: ProviderKind,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub mode: AnalysisMode,
}

impl From<AppSettings> for Settings {
    fn from(value: AppSettings) -> Self {
        Self {
            llm: LlmForm::from(value.llm),
            theme: value.theme,
            language: value.language,
        }
    }
}

impl From<Settings> for AppSettings {
    fn from(value: Settings) -> Self {
        Self {
            llm: LlmConfig::from(value.llm),
            theme: value.theme,
            language: value.language,
        }
    }
}

impl From<LlmConfig> for LlmForm {
    fn from(value: LlmConfig) -> Self {
        Self {
            provider: value.provider,
            api_key: value.api_key,
            endpoint: value.endpoint,
            model: value.model,
            temperature: value.temperature,
            mode: value.mode,
        }
    }
}

impl From<LlmForm> for LlmConfig {
    fn from(value: LlmForm) -> Self {
        Self {
            provider: value.provider,
            api_key: value.api_key,
            endpoint: value.endpoint,
            model: value.model,
            temperature: value.temperature,
            mode: value.mode,
        }
    }
}

/// Informations « À propos ».
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub struct About {
    pub version: String,
    pub name: String,
}

/// Mise à jour proposée par GitHub Releases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
    pub page_url: String,
    pub asset: Option<UpdateAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub struct UpdateAsset {
    pub name: String,
    pub url: String,
}

/// Progress du téléchargement d'un installeur.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub struct UpdateProgress {
    pub progress: u8,
}
