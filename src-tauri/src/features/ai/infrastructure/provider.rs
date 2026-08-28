//! Adaptateurs HTTP des fournisseurs supportés par les réglages historiques.

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::validation::is_local_or_private_ip;
use crate::features::ai::domain::{LlmConfig, ProviderKind};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::sync::Arc;

const MAX_RESPONSE: usize = 5 * 1024 * 1024;

#[async_trait]
pub trait LlmGenerator: Send + Sync {
    async fn generate(&self, prompt: &str, system: &str, json: bool) -> AppResult<String>;
    async fn test(&self) -> AppResult<()>;
    async fn list_models(&self) -> AppResult<Vec<String>>;
}

pub async fn build_provider(config: &LlmConfig) -> AppResult<Arc<dyn LlmGenerator>> {
    let endpoint = config.endpoint_effectif().trim_end_matches('/').to_owned();
    let url = url::Url::parse(&endpoint)
        .map_err(|_| AppError::Validation("Endpoint IA invalide".into()))?;
    let host = url
        .host_str()
        .ok_or_else(|| AppError::Validation("Endpoint IA sans hôte".into()))?;
    let port = url.port_or_known_default().unwrap_or(443);
    let mut builder = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(8))
        .timeout(std::time::Duration::from_secs(60 * 30))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Candilog/0.3");
    if !matches!(config.provider, ProviderKind::Ollama) {
        if url.scheme() != "https" {
            return Err(AppError::Validation(
                "Un endpoint IA distant doit utiliser HTTPS".into(),
            ));
        }
        let adresses: Vec<_> = tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| AppError::Validation("Impossible de résoudre l'endpoint IA".into()))?
            .collect();
        if adresses.is_empty() || adresses.iter().any(|a| is_local_or_private_ip(a.ip())) {
            return Err(AppError::Validation(
                "L'endpoint IA distant ne peut pas cibler le réseau local".into(),
            ));
        }
        builder = builder.resolve(host, adresses[0]);
    }
    let client = builder.build().map_err(AppError::from)?;
    Ok(Arc::new(ProviderHttp {
        config: config.clone(),
        endpoint,
        client,
    }))
}

struct ProviderHttp {
    config: LlmConfig,
    endpoint: String,
    client: reqwest::Client,
}

#[async_trait]
impl LlmGenerator for ProviderHttp {
    async fn generate(&self, prompt: &str, system: &str, json: bool) -> AppResult<String> {
        match self.config.provider {
            ProviderKind::Ollama => self.ollama(prompt, system, json).await,
            ProviderKind::Claude => self.claude(prompt, system).await,
            ProviderKind::Gemini => self.gemini(prompt, system, json).await,
            ProviderKind::OpenAI
            | ProviderKind::Mistral
            | ProviderKind::Nvidia
            | ProviderKind::Custom(_) => self.openai(prompt, system, json).await,
        }
    }

    async fn test(&self) -> AppResult<()> {
        self.list_models().await.map(|_| ())
    }

    async fn list_models(&self) -> AppResult<Vec<String>> {
        match self.config.provider {
            ProviderKind::Ollama => self.models_ollama().await,
            ProviderKind::Claude => self.models_claude().await,
            ProviderKind::Gemini => self.models_gemini().await,
            ProviderKind::OpenAI
            | ProviderKind::Mistral
            | ProviderKind::Nvidia
            | ProviderKind::Custom(_) => self.models_openai().await,
        }
    }
}

impl ProviderHttp {
    async fn ollama(&self, prompt: &str, system: &str, json: bool) -> AppResult<String> {
        let body = serde_json::json!({"model":self.config.model,"messages":[{"role":"system","content":system},{"role":"user","content":prompt}],"stream":false,"format":if json { serde_json::json!("json") } else { serde_json::Value::Null },"options":{"temperature":self.config.temperature}});
        let response = self
            .client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let value: serde_json::Value = json_limite(response).await?;
        text(&value, "/message/content")
    }

    async fn openai(&self, prompt: &str, system: &str, json: bool) -> AppResult<String> {
        let mut body = serde_json::json!({"model":self.config.model,"messages":[{"role":"system","content":system},{"role":"user","content":prompt}],"temperature":self.config.temperature});
        if json {
            body["response_format"] = serde_json::json!({"type":"json_object"});
        }
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .bearer_auth(self.config.api_key.as_deref().unwrap_or_default())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let value: serde_json::Value = json_limite(response).await?;
        text(&value, "/choices/0/message/content")
    }

    async fn claude(&self, prompt: &str, system: &str) -> AppResult<String> {
        let body = serde_json::json!({"model":self.config.model,"max_tokens":4096,"system":system,"messages":[{"role":"user","content":prompt}],"temperature":self.config.temperature});
        let response = self
            .client
            .post(format!("{}/v1/messages", self.endpoint))
            .header(
                "x-api-key",
                self.config.api_key.as_deref().unwrap_or_default(),
            )
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let value: serde_json::Value = json_limite(response).await?;
        text(&value, "/content/0/text")
    }

    async fn gemini(&self, prompt: &str, system: &str, json: bool) -> AppResult<String> {
        let body = serde_json::json!({"contents":[{"parts":[{"text":prompt}]}],"systemInstruction":{"parts":[{"text":system}]},"generationConfig":{"temperature":self.config.temperature,"responseMimeType":if json {"application/json"} else {"text/plain"}}});
        let response = self
            .client
            .post(format!(
                "{}/v1beta/models/{}:generateContent",
                self.endpoint, self.config.model
            ))
            .header(
                "x-goog-api-key",
                self.config.api_key.as_deref().unwrap_or_default(),
            )
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let value: serde_json::Value = json_limite(response).await?;
        text(&value, "/candidates/0/content/parts/0/text")
    }

    async fn models_ollama(&self) -> AppResult<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await?
            .error_for_status()?;
        let value: serde_json::Value = json_limite(response).await?;
        Ok(value
            .pointer("/models")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|model| model.get("name")?.as_str().map(str::to_owned))
            .collect())
    }

    async fn models_openai(&self) -> AppResult<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/v1/models", self.endpoint))
            .bearer_auth(self.config.api_key.as_deref().unwrap_or_default())
            .send()
            .await?
            .error_for_status()?;
        let value: serde_json::Value = json_limite(response).await?;
        Ok(value
            .pointer("/data")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|model| model.get("id")?.as_str().map(str::to_owned))
            .collect())
    }

    async fn models_claude(&self) -> AppResult<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/v1/models", self.endpoint))
            .header(
                "x-api-key",
                self.config.api_key.as_deref().unwrap_or_default(),
            )
            .header("anthropic-version", "2023-06-01")
            .send()
            .await?
            .error_for_status()?;
        let value: serde_json::Value = json_limite(response).await?;
        Ok(value
            .pointer("/data")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|model| model.get("id")?.as_str().map(str::to_owned))
            .collect())
    }

    async fn models_gemini(&self) -> AppResult<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/v1beta/models", self.endpoint))
            .header(
                "x-goog-api-key",
                self.config.api_key.as_deref().unwrap_or_default(),
            )
            .send()
            .await?
            .error_for_status()?;
        let value: serde_json::Value = json_limite(response).await?;
        Ok(value
            .pointer("/models")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|model| {
                let name = model.get("name")?.as_str()?;
                Some(name.strip_prefix("models/").unwrap_or(name).to_owned())
            })
            .collect())
    }
}

fn text(value: &serde_json::Value, pointer: &str) -> AppResult<String> {
    value
        .pointer(pointer)
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| AppError::Provider("Le fournisseur IA a renvoyé une réponse vide".into()))
}

async fn json_limite<T: DeserializeOwned>(mut response: reqwest::Response) -> AppResult<T> {
    if response
        .content_length()
        .is_some_and(|n| n > MAX_RESPONSE as u64)
    {
        return Err(AppError::Provider("Réponse IA trop volumineuse".into()));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len().saturating_add(chunk.len()) > MAX_RESPONSE {
            return Err(AppError::Provider("Réponse IA trop volumineuse".into()));
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(AppError::from)
}
