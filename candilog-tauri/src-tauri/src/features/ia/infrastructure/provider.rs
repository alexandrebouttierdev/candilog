//! Adaptateurs HTTP des fournisseurs supportés par les réglages historiques.

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::validation::is_local_or_private_ip;
use crate::features::ia::domain::{LlmConfig, ProviderKind};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::sync::Arc;

const MAX_RESPONSE: usize = 5 * 1024 * 1024;

#[async_trait]
pub trait GenerateurLlm: Send + Sync {
    async fn generer(&self, prompt: &str, systeme: &str, json: bool) -> AppResult<String>;
}

pub async fn construire_provider(config: &LlmConfig) -> AppResult<Arc<dyn GenerateurLlm>> {
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
impl GenerateurLlm for ProviderHttp {
    async fn generer(&self, prompt: &str, systeme: &str, json: bool) -> AppResult<String> {
        match self.config.provider {
            ProviderKind::Ollama => self.ollama(prompt, systeme, json).await,
            ProviderKind::Claude => self.claude(prompt, systeme).await,
            ProviderKind::Gemini => self.gemini(prompt, systeme, json).await,
            ProviderKind::OpenAI
            | ProviderKind::Mistral
            | ProviderKind::Nvidia
            | ProviderKind::Custom(_) => self.openai(prompt, systeme, json).await,
        }
    }
}

impl ProviderHttp {
    async fn ollama(&self, prompt: &str, systeme: &str, json: bool) -> AppResult<String> {
        let body = serde_json::json!({"model":self.config.model,"messages":[{"role":"system","content":systeme},{"role":"user","content":prompt}],"stream":false,"format":if json { serde_json::json!("json") } else { serde_json::Value::Null },"options":{"temperature":self.config.temperature}});
        let response = self
            .client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let value: serde_json::Value = json_limite(response).await?;
        texte(&value, "/message/content")
    }

    async fn openai(&self, prompt: &str, systeme: &str, json: bool) -> AppResult<String> {
        let mut body = serde_json::json!({"model":self.config.model,"messages":[{"role":"system","content":systeme},{"role":"user","content":prompt}],"temperature":self.config.temperature});
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
        texte(&value, "/choices/0/message/content")
    }

    async fn claude(&self, prompt: &str, systeme: &str) -> AppResult<String> {
        let body = serde_json::json!({"model":self.config.model,"max_tokens":4096,"system":systeme,"messages":[{"role":"user","content":prompt}],"temperature":self.config.temperature});
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
        texte(&value, "/content/0/text")
    }

    async fn gemini(&self, prompt: &str, systeme: &str, json: bool) -> AppResult<String> {
        let body = serde_json::json!({"contents":[{"parts":[{"text":prompt}]}],"systemInstruction":{"parts":[{"text":systeme}]},"generationConfig":{"temperature":self.config.temperature,"responseMimeType":if json {"application/json"} else {"text/plain"}}});
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
        texte(&value, "/candidates/0/content/parts/0/text")
    }
}

fn texte(value: &serde_json::Value, pointer: &str) -> AppResult<String> {
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
