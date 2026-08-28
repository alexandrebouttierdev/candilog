//! Configuration compatible avec le JSON historique de `parametres`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Ollama,
    Claude,
    OpenAI,
    Gemini,
    Mistral,
    Nvidia,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: ProviderKind,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub model: String,
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::Ollama,
            api_key: None,
            endpoint: Some("http://localhost:11434".into()),
            model: "llama3.2:3b".into(),
            temperature: 0.7,
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
pub struct ParametresStockes {
    /// Absent sur une base neuve (`data = '{}'`), présent dès que les réglages ont été sauvés.
    #[serde(default)]
    pub llm: LlmConfig,
}
