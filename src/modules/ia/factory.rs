//! Construction d'un fournisseur LLM à partir de la configuration.

use crate::modules::ia::provider::LlmProvider;
use crate::modules::ia::providers::{
    ClaudeProvider, GeminiProvider, OllamaProvider, OpenAiCompatProvider,
};
use crate::shared::llm::{LlmConfig, ProviderKind};
use std::sync::Arc;

/// Endpoint par défaut d'un fournisseur (utilisé si `LlmConfig.endpoint` est absent).
#[must_use]
pub fn default_endpoint(kind: &ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Ollama => "http://localhost:11434",
        ProviderKind::Claude => "https://api.anthropic.com",
        ProviderKind::OpenAI | ProviderKind::Custom(_) => "https://api.openai.com",
        ProviderKind::Mistral => "https://api.mistral.ai",
        ProviderKind::Nvidia => "https://integrate.api.nvidia.com",
        ProviderKind::Gemini => "https://generativelanguage.googleapis.com",
    }
}

/// Construit le fournisseur LLM correspondant à la configuration.
#[must_use]
pub fn build_provider(config: &LlmConfig) -> Arc<dyn LlmProvider> {
    build_provider_pinned(config, None)
}

/// Variante épinglée sur l'adresse validée par `validate_llm_endpoint`.
#[must_use]
pub fn build_provider_pinned(
    config: &LlmConfig,
    pin: Option<crate::shared::llm::EndpointPin>,
) -> Arc<dyn LlmProvider> {
    let endpoint = config
        .endpoint
        .clone()
        .unwrap_or_else(|| default_endpoint(&config.provider).to_string());
    let key = config.api_key.clone().unwrap_or_default();
    let model = config.model.clone();
    let temp = config.temperature;
    match config.provider {
        ProviderKind::Ollama => Arc::new(OllamaProvider::new(endpoint, model, temp, pin)),
        ProviderKind::OpenAI
        | ProviderKind::Mistral
        | ProviderKind::Nvidia
        | ProviderKind::Custom(_) => {
            Arc::new(OpenAiCompatProvider::new(endpoint, key, model, temp, pin))
        }
        ProviderKind::Claude => Arc::new(ClaudeProvider::new(endpoint, key, model, temp, pin)),
        ProviderKind::Gemini => Arc::new(GeminiProvider::new(endpoint, key, model, temp, pin)),
    }
}

#[cfg(test)]
#[path = "tests/factory/mod.rs"]
mod tests;
