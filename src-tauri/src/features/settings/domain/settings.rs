//! Modèle persisté (JSON Iced) et DTO IPC camelCase.

use crate::features::ai::domain::{
    AiRoutes, AnalysisMode, LlmConfig, ManagedOllamaSettings, ProviderKind,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

/// Empreinte d'un fournisseur sans secret — une entrée par fournisseur dans `llm_presets`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LlmProviderPreset {
    pub endpoint: Option<String>,
    pub model: String,
    pub temperature: f32,
    #[serde(default)]
    pub mode: AnalysisMode,
}

impl LlmProviderPreset {
    #[must_use]
    pub fn from_config(config: &LlmConfig) -> Self {
        Self {
            endpoint: config.endpoint.clone(),
            model: config.model.clone(),
            temperature: config.temperature,
            mode: config.mode,
        }
    }

    #[must_use]
    pub fn endpoint_effectif(&self, provider: &ProviderKind) -> String {
        self.endpoint
            .clone()
            .unwrap_or_else(|| default_endpoint(provider).unwrap_or_default())
    }
}

/// Forme IPC d'un preset, avec l'état de clé (jamais le secret).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub struct LlmProviderPresetForm {
    pub endpoint: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub mode: AnalysisMode,
    pub api_key_configured: bool,
}

impl From<LlmProviderPresetForm> for LlmProviderPreset {
    fn from(value: LlmProviderPresetForm) -> Self {
        Self {
            endpoint: value.endpoint,
            model: value.model,
            temperature: value.temperature,
            mode: value.mode,
        }
    }
}

/// Paramètres complets tels qu'ils sont écrits dans `parametres.data`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub llm: LlmConfig,
    /// Configurations mémorisées par fournisseur (`openai`, `mistral`, …).
    #[serde(default)]
    pub llm_presets: BTreeMap<String, LlmProviderPreset>,
    #[serde(default)]
    pub managed_ollama: ManagedOllamaSettings,
    /// Routage des tâches IA ; une tâche absente suit le fournisseur principal.
    #[serde(default)]
    pub ai_routes: AiRoutes,
    #[serde(default)]
    pub theme: ThemePref,
    #[serde(default = "language_fr")]
    pub language: String,
}

fn language_fr() -> String {
    "fr".into()
}

#[must_use]
pub fn default_endpoint(provider: &ProviderKind) -> Option<String> {
    match provider {
        ProviderKind::CandilogLocal => None,
        ProviderKind::Ollama => Some("http://localhost:11434".into()),
        ProviderKind::Claude => Some("https://api.anthropic.com".into()),
        ProviderKind::Gemini => Some("https://generativelanguage.googleapis.com".into()),
        ProviderKind::Mistral => Some("https://api.mistral.ai".into()),
        ProviderKind::DeepSeek => Some("https://api.deepseek.com".into()),
        ProviderKind::OpenAI | ProviderKind::Custom(_) => Some("https://api.openai.com".into()),
    }
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
            llm_presets: BTreeMap::new(),
            managed_ollama: ManagedOllamaSettings::default(),
            ai_routes: AiRoutes::new(),
            theme: ThemePref::System,
            language: language_fr(),
        }
    }
}

impl AppSettings {
    /// Mémorise la configuration active sous l'identifiant de son fournisseur.
    pub fn capture_llm_preset(&mut self) {
        let id = self.llm.provider.storage_id().to_string();
        self.llm_presets
            .insert(id, LlmProviderPreset::from_config(&self.llm));
    }
}

/// Forme IPC destinée à React.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub struct Settings {
    pub llm: LlmForm,
    #[serde(default)]
    pub llm_presets: BTreeMap<String, LlmProviderPresetForm>,
    /// Routage des tâches IA ; une tâche absente suit le fournisseur principal.
    #[serde(default)]
    pub ai_routes: AiRoutes,
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
        Self::from_app(value, false, &BTreeMap::new())
    }
}

impl Settings {
    #[must_use]
    pub fn from_app(
        value: AppSettings,
        api_key_configured: bool,
        keys_configured: &BTreeMap<String, bool>,
    ) -> Self {
        let llm_presets = value
            .llm_presets
            .iter()
            .map(|(id, preset)| {
                (
                    id.clone(),
                    LlmProviderPresetForm {
                        endpoint: preset.endpoint.clone(),
                        model: preset.model.clone(),
                        temperature: preset.temperature,
                        mode: preset.mode,
                        api_key_configured: keys_configured.get(id).copied().unwrap_or(false),
                    },
                )
            })
            .collect();
        Self {
            llm: LlmForm::from_config(value.llm, api_key_configured),
            llm_presets,
            ai_routes: value.ai_routes,
            theme: value.theme,
            language: value.language,
        }
    }
}

impl From<Settings> for AppSettings {
    fn from(value: Settings) -> Self {
        let mut app = Self {
            llm: LlmConfig::from(value.llm),
            llm_presets: value
                .llm_presets
                .into_iter()
                .map(|(id, preset)| (id, LlmProviderPreset::from(preset)))
                .collect(),
            managed_ollama: ManagedOllamaSettings::default(),
            ai_routes: value.ai_routes,
            theme: value.theme,
            language: value.language,
        };
        app.capture_llm_preset();
        app
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
