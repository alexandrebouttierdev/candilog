//! Modèle persisté (JSON Iced) et DTO IPC camelCase.

use crate::features::ia::domain::{AnalysisMode, LlmConfig, ProviderKind};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Préférence de thème, identique à l'enum historique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "parametres.ts")]
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
    #[serde(default = "langue_fr")]
    pub langue: String,
}

fn langue_fr() -> String {
    "fr".into()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            theme: ThemePref::System,
            langue: langue_fr(),
        }
    }
}

/// Forme IPC destinée à React.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "parametres.ts")]
pub struct Parametres {
    pub llm: LlmFormulaire,
    pub theme: ThemePref,
    pub langue: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "parametres.ts")]
pub struct LlmFormulaire {
    pub provider: ProviderKind,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub mode: AnalysisMode,
}

impl From<AppSettings> for Parametres {
    fn from(value: AppSettings) -> Self {
        Self {
            llm: LlmFormulaire::from(value.llm),
            theme: value.theme,
            langue: value.langue,
        }
    }
}

impl From<Parametres> for AppSettings {
    fn from(value: Parametres) -> Self {
        Self {
            llm: LlmConfig::from(value.llm),
            theme: value.theme,
            langue: value.langue,
        }
    }
}

impl From<LlmConfig> for LlmFormulaire {
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

impl From<LlmFormulaire> for LlmConfig {
    fn from(value: LlmFormulaire) -> Self {
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
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "parametres.ts")]
pub struct APropos {
    pub version: String,
    pub nom: String,
}

/// Mise à jour proposée par GitHub Releases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "parametres.ts")]
pub struct MiseAJour {
    pub version: String,
    pub notes: String,
    pub page_url: String,
    pub asset: Option<AssetMaj>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "parametres.ts")]
pub struct AssetMaj {
    pub nom: String,
    pub url: String,
}

/// Progression du téléchargement d'un installeur.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "parametres.ts")]
pub struct ProgressionMaj {
    pub progression: u8,
}
