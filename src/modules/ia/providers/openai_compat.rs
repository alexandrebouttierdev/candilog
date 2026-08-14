//! Fournisseur compatible `OpenAI` (`OpenAI`, Mistral, Custom) : `POST /v1/chat/completions`.

use crate::modules::ia::provider::{provider_err, sse_data, GenOptions, LlmProvider};
use crate::shared::error::AppResult;
use crate::shared::http::{
    json_limited, read_lines_stream, MAX_PROVIDER_RESPONSE_BYTES, PROVIDER_GENERATION_TIMEOUT,
};
use async_trait::async_trait;
use serde::Deserialize;

/// Fournisseur au format `OpenAI` Chat Completions.
pub struct OpenAiCompatProvider {
    endpoint: String,
    api_key: String,
    model: String,
    temperature: f32,
    http: reqwest::Client,
}

impl OpenAiCompatProvider {
    /// Construit le fournisseur compatible `OpenAI`.
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

    /// Appelle `/v1/chat/completions`, en bornant la sortie à `max_tokens` si fourni.
    async fn complete(
        &self,
        prompt: &str,
        system: &str,
        max_tokens: Option<u32>,
    ) -> AppResult<String> {
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt},
            ],
            "temperature": self.temperature,
        });
        if let Some(max_tokens) = max_tokens {
            body["max_tokens"] = max_tokens.into();
        }
        let response = self
            .http
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .timeout(PROVIDER_GENERATION_TIMEOUT)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        let resp: ChatResponse = json_limited(response, MAX_PROVIDER_RESPONSE_BYTES)
            .await
            .map_err(provider_err)?;
        resp.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| provider_err("réponse sans choix"))
    }
}

/// Message d'un choix de complétion.
#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

/// Choix de complétion.
#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

/// Réponse `/v1/chat/completions`.
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

/// Élément de la liste `/v1/models`.
#[derive(Deserialize)]
struct ModelEntry {
    id: String,
}

/// Réponse `/v1/models`.
#[derive(Deserialize)]
struct ModelsList {
    data: Vec<ModelEntry>,
}

#[async_trait]
impl LlmProvider for OpenAiCompatProvider {
    async fn health_check(&self) -> AppResult<()> {
        self.http
            .get(format!("{}/v1/models", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
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
        // `num_ctx` n'a pas d'équivalent (contexte géré côté serveur) ; `num_predict` borne
        // la sortie via `max_tokens`. Le schéma n'est pas transmis (ces modèles sont fiables
        // en JSON via le prompt + réparation du moteur).
        self.complete(prompt, system, options.num_predict).await
    }

    async fn stream(
        &self,
        prompt: &str,
        system: &str,
        options: &GenOptions,
        on_chunk: &mut (dyn FnMut(String) + Send),
    ) -> AppResult<String> {
        // `stream: true` → SSE : chaque `data:` porte un `choices[0].delta.content`.
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt},
            ],
            "temperature": self.temperature,
            "stream": true,
        });
        if let Some(max_tokens) = options.num_predict {
            body["max_tokens"] = max_tokens.into();
        }
        let response = self
            .http
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .timeout(PROVIDER_GENERATION_TIMEOUT)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        // Certains endpoints compatibles ignorent `stream: true` et renvoient une
        // complétion `JSON` classique : sans ce repli, le flux `SSE` était lu ligne à
        // ligne, aucun `data:` n'était reconnu, et la lettre sortait vide.
        let est_json = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|valeur| valeur.to_str().ok())
            .is_some_and(|valeur| valeur.contains("application/json"));
        if est_json {
            let reponse: ChatResponse = json_limited(response, MAX_PROVIDER_RESPONSE_BYTES)
                .await
                .map_err(provider_err)?;
            let contenu = reponse
                .choices
                .into_iter()
                .next()
                .map(|choix| choix.message.content)
                .ok_or_else(|| provider_err("réponse sans choix"))?;
            on_chunk(contenu.clone());
            return Ok(contenu);
        }
        let mut full = String::new();
        read_lines_stream(response, |line| {
            if let Some(data) = sse_data(line) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                    let delta = v
                        .pointer("/choices/0/delta/content")
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
        if full.is_empty() {
            // Le flux SSE n'a produit aucun contenu (format inattendu) : repli sur la
            // complétion classique, qui reste fonctionnelle sur tous les endpoints.
            let contenu = self.complete(prompt, system, options.num_predict).await?;
            on_chunk(contenu.clone());
            return Ok(contenu);
        }
        Ok(full)
    }

    async fn list_models(&self) -> AppResult<Vec<String>> {
        let response = self
            .http
            .get(format!("{}/v1/models", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        let resp: ModelsList = json_limited(response, MAX_PROVIDER_RESPONSE_BYTES)
            .await
            .map_err(provider_err)?;
        Ok(resp.data.into_iter().map(|m| m.id).collect())
    }
}

#[cfg(test)]
#[path = "tests/openai_compat/mod.rs"]
mod tests;
