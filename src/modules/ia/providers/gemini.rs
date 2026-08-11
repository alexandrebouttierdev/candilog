//! Fournisseur Google Gemini : `POST /v1beta/models/{model}:generateContent`.

use crate::modules::ia::provider::{provider_err, sse_data, GenOptions, LlmProvider};
use crate::shared::error::AppResult;
use crate::shared::http::{
    json_limited, read_lines_stream, MAX_PROVIDER_RESPONSE_BYTES, PROVIDER_GENERATION_TIMEOUT,
};
use async_trait::async_trait;
use serde::Deserialize;

/// Fournisseur Gemini.
pub struct GeminiProvider {
    endpoint: String,
    api_key: String,
    model: String,
    temperature: f32,
    http: reqwest::Client,
}

impl GeminiProvider {
    /// Construit le fournisseur Gemini.
    #[must_use]
    pub fn new(
        endpoint: String,
        api_key: String,
        model: String,
        temperature: f32,
        pin: Option<crate::shared::llm::EndpointPin>,
    ) -> Self {
        Self {
            endpoint,
            api_key,
            model,
            temperature,
            http: crate::shared::http::client_pinned(pin.as_ref()),
        }
    }

    /// Appelle `generateContent` en bornant la sortie à `max_tokens` si fourni.
    async fn complete(
        &self,
        prompt: &str,
        system: &str,
        max_tokens: Option<u32>,
    ) -> AppResult<String> {
        let url = format!(
            "{}/v1beta/models/{}:generateContent",
            self.endpoint, self.model
        );
        let mut generation_config = serde_json::json!({"temperature": self.temperature});
        if let Some(max_tokens) = max_tokens {
            generation_config["maxOutputTokens"] = max_tokens.into();
        }
        let body = serde_json::json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "systemInstruction": {"parts": [{"text": system}]},
            "generationConfig": generation_config,
        });
        let response = self
            .http
            .post(url)
            .timeout(PROVIDER_GENERATION_TIMEOUT)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        let resp: GeminiResponse = json_limited(response, MAX_PROVIDER_RESPONSE_BYTES)
            .await
            .map_err(provider_err)?;
        resp.candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| provider_err("réponse sans candidat"))
    }
}

/// Partie de texte d'un candidat.
#[derive(Deserialize)]
struct GeminiPart {
    text: String,
}

/// Contenu d'un candidat.
#[derive(Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

/// Candidat de réponse.
#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

/// Réponse `generateContent`.
#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

/// Élément de la liste `/v1beta/models`.
#[derive(Deserialize)]
struct GeminiModelEntry {
    name: String,
}

/// Réponse `/v1beta/models`.
#[derive(Deserialize)]
struct GeminiModelsList {
    models: Vec<GeminiModelEntry>,
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn health_check(&self) -> AppResult<()> {
        self.http
            .get(format!("{}/v1beta/models", self.endpoint))
            .header("x-goog-api-key", &self.api_key)
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
        // `num_ctx` sans objet (contexte serveur) ; `num_predict` borne `maxOutputTokens`.
        self.complete(prompt, system, options.num_predict).await
    }

    async fn stream(
        &self,
        prompt: &str,
        system: &str,
        options: &GenOptions,
        on_chunk: &mut (dyn FnMut(String) + Send),
    ) -> AppResult<String> {
        // `streamGenerateContent?alt=sse` → SSE : chaque `data:` porte un candidat partiel.
        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse",
            self.endpoint, self.model
        );
        let mut generation_config = serde_json::json!({"temperature": self.temperature});
        if let Some(max_tokens) = options.num_predict {
            generation_config["maxOutputTokens"] = max_tokens.into();
        }
        let body = serde_json::json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "systemInstruction": {"parts": [{"text": system}]},
            "generationConfig": generation_config,
        });
        let response = self
            .http
            .post(url)
            .timeout(PROVIDER_GENERATION_TIMEOUT)
            .header("x-goog-api-key", &self.api_key)
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
                        .pointer("/candidates/0/content/parts/0/text")
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

    #[allow(clippy::unnecessary_lazy_evaluations)]
    async fn list_models(&self) -> AppResult<Vec<String>> {
        let response = self
            .http
            .get(format!("{}/v1beta/models", self.endpoint))
            .header("x-goog-api-key", &self.api_key)
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        let resp: GeminiModelsList = json_limited(response, MAX_PROVIDER_RESPONSE_BYTES)
            .await
            .map_err(provider_err)?;
        Ok(resp
            .models
            .into_iter()
            .map(|m| {
                m.name
                    .strip_prefix("models/")
                    .unwrap_or_else(|| m.name.as_str())
                    .to_string()
            })
            .collect())
    }
}

#[cfg(test)]
#[path = "tests/gemini/mod.rs"]
mod tests;
