//! Fournisseur Anthropic Claude (Messages API) : `POST /v1/messages`.

use crate::modules::ia::provider::{provider_err, sse_data, GenOptions, LlmProvider};
use crate::shared::error::AppResult;
use crate::shared::http::{
    json_limited, read_lines_stream, MAX_PROVIDER_RESPONSE_BYTES, PROVIDER_GENERATION_TIMEOUT,
};
use async_trait::async_trait;
use serde::Deserialize;

/// Nombre maximal de tokens générés par défaut (l'API Claude l'exige, borne haute).
const MAX_TOKENS: u32 = 4096;

/// Fournisseur Claude (Messages API).
pub struct ClaudeProvider {
    endpoint: String,
    api_key: String,
    model: String,
    temperature: f32,
    http: reqwest::Client,
}

impl ClaudeProvider {
    /// Construit le fournisseur Claude.
    #[must_use]
    pub fn new(endpoint: String, api_key: String, model: String, temperature: f32) -> Self {
        Self {
            endpoint,
            api_key,
            model,
            temperature,
            http: crate::shared::http::client(),
        }
    }

    /// Appelle `/v1/messages` en bornant la sortie à `max_tokens` (défaut [`MAX_TOKENS`]).
    #[allow(clippy::unnecessary_lazy_evaluations)]
    async fn complete(
        &self,
        prompt: &str,
        system: &str,
        max_tokens: Option<u32>,
    ) -> AppResult<String> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": max_tokens.unwrap_or_else(|| MAX_TOKENS),
            "system": system,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": self.temperature,
        });
        let response = self
            .http
            .post(format!("{}/v1/messages", self.endpoint))
            .timeout(PROVIDER_GENERATION_TIMEOUT)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        let resp: ClaudeResponse = json_limited(response, MAX_PROVIDER_RESPONSE_BYTES)
            .await
            .map_err(provider_err)?;
        resp.content
            .into_iter()
            .next()
            .map(|c| c.text)
            .ok_or_else(|| provider_err("réponse sans contenu"))
    }
}

/// Bloc de contenu renvoyé par Claude.
#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

/// Réponse `/v1/messages`.
#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

/// Élément de la liste `/v1/models`.
#[derive(Deserialize)]
struct ClaudeModelEntry {
    id: String,
}

/// Réponse `/v1/models`.
#[derive(Deserialize)]
struct ClaudeModelsList {
    data: Vec<ClaudeModelEntry>,
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn health_check(&self) -> AppResult<()> {
        self.http
            .get(format!("{}/v1/models", self.endpoint))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        Ok(())
    }

    async fn generate(&self, prompt: &str, system: &str) -> AppResult<String> {
        self.complete(prompt, system, None).await
    }

    async fn generate_with_options(
        &self,
        prompt: &str,
        system: &str,
        _schema: Option<&serde_json::Value>,
        options: &GenOptions,
    ) -> AppResult<String> {
        // `num_ctx` sans objet (contexte serveur) ; `num_predict` borne `max_tokens`.
        self.complete(prompt, system, options.num_predict).await
    }

    #[allow(clippy::unnecessary_lazy_evaluations)]
    async fn stream(
        &self,
        prompt: &str,
        system: &str,
        options: &GenOptions,
        on_chunk: &mut (dyn FnMut(String) + Send),
    ) -> AppResult<String> {
        // `stream: true` → SSE Anthropic : les fragments arrivent en `content_block_delta` (`delta.text`).
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": options.num_predict.unwrap_or_else(|| MAX_TOKENS),
            "system": system,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": self.temperature,
            "stream": true,
        });
        let response = self
            .http
            .post(format!("{}/v1/messages", self.endpoint))
            .timeout(PROVIDER_GENERATION_TIMEOUT)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        let mut full = String::new();
        read_lines_stream(response, |line| {
            if let Some(data) = sse_data(line) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                    let delta = v
                        .pointer("/delta/text")
                        .and_then(|c| c.as_str())
                        .map(str::to_string);
                    if let Some(txt) = delta {
                        if !txt.is_empty() {
                            full.push_str(&txt);
                            on_chunk(txt);
                        }
                    }
                }
            }
        })
        .await?;
        Ok(full)
    }

    async fn list_models(&self) -> AppResult<Vec<String>> {
        let response = self
            .http
            .get(format!("{}/v1/models", self.endpoint))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        let resp: ClaudeModelsList = json_limited(response, MAX_PROVIDER_RESPONSE_BYTES)
            .await
            .map_err(provider_err)?;
        Ok(resp.data.into_iter().map(|m| m.id).collect())
    }
}

#[cfg(test)]
#[path = "tests/claude/mod.rs"]
mod tests;
