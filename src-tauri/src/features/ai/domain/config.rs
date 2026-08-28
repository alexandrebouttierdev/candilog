//! Configuration compatible avec le JSON historique de `parametres`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Mode d'analyse, rétro-compatible avec les bases antérieures à son introduction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub enum AnalysisMode {
    #[default]
    Auto,
    Small,
    Standard,
    Advanced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub enum ProviderKind {
    Ollama,
    Claude,
    #[serde(rename = "openai", alias = "open_ai", alias = "OpenAI")]
    OpenAI,
    Gemini,
    Mistral,
    Nvidia,
    Custom(String),
}

/// Champs en snake_case : c'est le JSON persisté par l'application Iced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: ProviderKind,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub model: String,
    pub temperature: f32,
    #[serde(default)]
    pub mode: AnalysisMode,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::Ollama,
            api_key: None,
            endpoint: Some("http://localhost:11434".into()),
            model: "llama3.2:3b".into(),
            temperature: 0.7,
            mode: AnalysisMode::Auto,
        }
    }
}

impl LlmConfig {
    #[must_use]
    pub fn endpoint_effectif(&self) -> &str {
        self.endpoint.as_deref().unwrap_or(match self.provider {
            ProviderKind::Ollama => "http://localhost:11434",
            ProviderKind::Claude => "https://api.anthropic.com",
            ProviderKind::Gemini => "https://generativelanguage.googleapis.com",
            ProviderKind::Mistral => "https://api.mistral.ai",
            ProviderKind::Nvidia => "https://integrate.api.nvidia.com",
            ProviderKind::OpenAI | ProviderKind::Custom(_) => "https://api.openai.com",
        })
    }

    #[must_use]
    pub fn est_configure(&self) -> bool {
        !self.model.trim().is_empty()
            && match self.provider {
                ProviderKind::Ollama => !self.endpoint_effectif().trim().is_empty(),
                _ => self
                    .api_key
                    .as_deref()
                    .is_some_and(|key| !key.trim().is_empty()),
            }
    }
}

#[derive(Deserialize)]
pub struct SettingsStockes {
    /// Absent sur une base neuve (`data = '{}'`), présent dès que les réglages ont été sauvés.
    #[serde(default)]
    pub llm: LlmConfig,
}
