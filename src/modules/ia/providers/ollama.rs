//! Fournisseur Ollama (local).

use crate::modules::ia::provider::{provider_err, GenOptions, LlmProvider};
use crate::shared::error::AppResult;
use crate::shared::http::{
    json_limited, read_lines_stream, MAX_PROVIDER_RESPONSE_BYTES, PROVIDER_GENERATION_TIMEOUT,
};
use async_trait::async_trait;
use serde::Deserialize;

/// Fournisseur Ollama : `POST /api/chat`, santé via `GET /api/tags`.
pub struct OllamaProvider {
    endpoint: String,
    model: String,
    temperature: f32,
    http: reqwest::Client,
}

/// Indique si le modèle appartient à une famille « à raisonnement » (chaîne de pensée).
///
/// Ces modèles (gpt-oss, `DeepSeek-R1`, Qwen3, Magistral, `QwQ`…) émettent par défaut une longue
/// réflexion avant leur réponse. Pour nos extractions JSON c'est inutile et coûteux : cela
/// ralentit chaque appel et, quand `num_predict` est borné, la réflexion peut épuiser le budget
/// de sortie avant le JSON — d'où une sortie invalide puis des reprises. On force donc
/// `think: false` pour eux. Les autres modèles ne supportent pas le paramètre `think` (Ollama
/// répondrait 400) : on ne l'envoie que pour ces familles connues.
fn model_disables_thinking(model: &str) -> bool {
    let m = model.to_lowercase();
    [
        "gpt-oss",
        "deepseek-r1",
        "qwen3",
        "magistral",
        "qwq",
        "cogito",
        "exaone-deep",
        "smallthinker",
        "marco-o1",
        "phi4-reasoning",
        "reasoning",
    ]
    .iter()
    .any(|needle| m.contains(needle))
}

impl OllamaProvider {
    /// Construit le fournisseur Ollama.
    #[must_use]
    pub fn new(
        endpoint: String,
        model: String,
        temperature: f32,
        pin: Option<crate::shared::llm::EndpointPin>,
    ) -> Self {
        Self {
            endpoint,
            model,
            temperature,
            http: crate::shared::http::client_pinned(pin.as_ref()),
        }
    }

    /// Appelle `/api/chat` avec la contrainte de sortie `format` fournie
    /// (`"json"` ou un schéma `JSON` complet — décodage guidé par grammaire) et les
    /// options d'exécution (`num_ctx`, `num_predict`, `keep_alive`).
    async fn chat(
        &self,
        prompt: &str,
        system: &str,
        format: serde_json::Value,
        opts: &GenOptions,
    ) -> AppResult<String> {
        // `options` contient toujours la température ; on n'y ajoute `num_ctx`/`num_predict`
        // que s'ils sont fournis (sinon Ollama applique ses valeurs par défaut).
        let mut options = serde_json::json!({"temperature": self.temperature});
        if let Some(num_ctx) = opts.num_ctx {
            options["num_ctx"] = num_ctx.into();
        }
        if let Some(num_predict) = opts.num_predict {
            options["num_predict"] = num_predict.into();
        }
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt},
            ],
            "stream": false,
            "format": format,
            "options": options,
        });
        // `keep_alive` maintient le modèle chargé entre appels séquentiels (évite de recharger
        // les poids à chaque étape — poste de latence dominant sur un modèle local).
        if let Some(keep_alive) = opts.keep_alive {
            body["keep_alive"] = keep_alive.into();
        }
        // Coupe la chaîne de pensée des modèles à raisonnement : pour une extraction JSON,
        // elle est inutile, lente, et risque de faire dépasser `num_predict` avant le JSON.
        if model_disables_thinking(&self.model) {
            body["think"] = false.into();
        }
        let response = self
            .http
            .post(format!("{}/api/chat", self.endpoint))
            .timeout(PROVIDER_GENERATION_TIMEOUT)
            .json(&body)
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        let resp: OllamaResponse = json_limited(response, MAX_PROVIDER_RESPONSE_BYTES)
            .await
            .map_err(provider_err)?;
        Ok(resp.message.content)
    }
}

/// Message renvoyé par Ollama.
#[derive(Deserialize)]
struct OllamaMessage {
    content: String,
}

/// Réponse `/api/chat` d'Ollama.
#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

/// Élément de la liste `/api/tags`.
#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

/// Réponse `/api/tags`.
#[derive(Deserialize)]
struct OllamaTags {
    models: Vec<OllamaModel>,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn health_check(&self) -> AppResult<()> {
        self.http
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        Ok(())
    }

    async fn generate(&self, prompt: &str, system: &str) -> AppResult<String> {
        // Toutes les opérations exposées par `LlmProvider::generate` dans
        // Candilog attendent un objet JSON. La contrainte native d'Ollama
        // évite les sorties tronquées ou ponctuées incorrectement des
        // petits modèles, avant même la réparation défensive du moteur.
        self.chat(
            prompt,
            system,
            serde_json::json!("json"),
            &GenOptions::none(),
        )
        .await
    }

    async fn generate_structured(
        &self,
        prompt: &str,
        system: &str,
        schema: &serde_json::Value,
    ) -> AppResult<String> {
        // Ollama accepte un schéma JSON complet dans `format` : le décodage est
        // alors contraint par grammaire, le format de sortie est garanti.
        self.chat(prompt, system, schema.clone(), &GenOptions::none())
            .await
    }

    async fn generate_with_options(
        &self,
        prompt: &str,
        system: &str,
        schema: Option<&serde_json::Value>,
        options: &GenOptions,
    ) -> AppResult<String> {
        // `format` = schéma complet si fourni (décodage contraint par grammaire), sinon `"json"`.
        let format = schema.cloned().unwrap_or_else(|| serde_json::json!("json"));
        self.chat(prompt, system, format, options).await
    }

    async fn stream(
        &self,
        prompt: &str,
        system: &str,
        options: &GenOptions,
        on_chunk: &mut (dyn FnMut(String) + Send),
    ) -> AppResult<String> {
        // `stream: true`, sortie **texte brut** (pas de `format` JSON) : chaque ligne NDJSON porte
        // un fragment `message.content` cumulable directement.
        let mut opts = serde_json::json!({"temperature": self.temperature});
        if let Some(num_ctx) = options.num_ctx {
            opts["num_ctx"] = num_ctx.into();
        }
        if let Some(num_predict) = options.num_predict {
            opts["num_predict"] = num_predict.into();
        }
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt},
            ],
            "stream": true,
            "options": opts,
        });
        if let Some(keep_alive) = options.keep_alive {
            body["keep_alive"] = keep_alive.into();
        }
        if model_disables_thinking(&self.model) {
            body["think"] = false.into();
        }
        let response = self
            .http
            .post(format!("{}/api/chat", self.endpoint))
            .timeout(PROVIDER_GENERATION_TIMEOUT)
            .json(&body)
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        let mut full = String::new();
        read_lines_stream(response, |line| {
            if line.is_empty() {
                return;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                let delta = v
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .map(str::to_string);
                if let Some(txt) = delta {
                    if !txt.is_empty() {
                        full.push_str(&txt);
                        on_chunk(txt);
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
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
            .map_err(provider_err)?
            .error_for_status()
            .map_err(provider_err)?;
        let resp: OllamaTags = json_limited(response, MAX_PROVIDER_RESPONSE_BYTES)
            .await
            .map_err(provider_err)?;
        Ok(resp.models.into_iter().map(|m| m.name).collect())
    }
}

#[cfg(test)]
#[path = "tests/ollama/mod.rs"]
mod tests;
