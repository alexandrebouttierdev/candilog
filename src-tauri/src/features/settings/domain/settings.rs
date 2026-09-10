//! Modèle persisté (JSON Iced) et DTO IPC camelCase.

use crate::features::ai::domain::{AnalysisMode, LlmConfig, ManagedOllamaSettings, ProviderKind};
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
    pub managed_ollama: ManagedOllamaSettings,
    #[serde(default)]
    pub theme: ThemePref,
    #[serde(default = "language_fr")]
    pub language: String,
}

fn language_fr() -> String {
    "fr".into()
}

/// Migre les anciens réglages (`mistral_local`, bloc `local_ai`) avant désérialisation.
#[must_use]
pub fn preparer_settings_json(text: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(text) else {
        return text.to_string();
    };
    if let Some(object) = value.as_object_mut() {
        object.remove("local_ai");
        if let Some(llm) = object
            .get_mut("llm")
            .and_then(|entry| entry.as_object_mut())
        {
            if llm.get("provider") == Some(&serde_json::Value::String("mistral_local".into())) {
                llm.insert("provider".into(), serde_json::json!("candilog_local"));
                llm.insert("model".into(), serde_json::json!(""));
                llm.insert("endpoint".into(), serde_json::Value::Null);
            }
        }
    }
    serde_json::to_string(&value).unwrap_or_else(|_| text.to_string())
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            managed_ollama: ManagedOllamaSettings::default(),
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
    pub api_key_configured: bool,
    pub endpoint: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub mode: AnalysisMode,
}

impl From<AppSettings> for Settings {
    fn from(value: AppSettings) -> Self {
        Self::from_app(value, false)
    }
}

impl Settings {
    #[must_use]
    pub fn from_app(value: AppSettings, api_key_configured: bool) -> Self {
        Self {
            llm: LlmForm::from_config(value.llm, api_key_configured),
            theme: value.theme,
            language: value.language,
        }
    }
}

impl From<Settings> for AppSettings {
    fn from(value: Settings) -> Self {
        Self {
            llm: LlmConfig::from(value.llm),
            managed_ollama: ManagedOllamaSettings::default(),
            theme: value.theme,
            language: value.language,
        }
    }
}

impl From<LlmConfig> for LlmForm {
    fn from(value: LlmConfig) -> Self {
        Self::from_config(value, false)
    }
}

impl LlmForm {
    #[must_use]
    pub fn from_config(value: LlmConfig, api_key_configured: bool) -> Self {
        Self {
            provider: value.provider,
            api_key_configured,
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
            api_key: None,
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

/// Résultat détaillé d'une réinitialisation locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub struct ResetOutcome {
    pub data_cleared: bool,
    pub secret_cleared: bool,
}
