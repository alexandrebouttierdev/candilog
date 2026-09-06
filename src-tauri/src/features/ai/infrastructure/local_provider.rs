//! Adaptateur `LlmGenerator` du runtime embarqué Mistral Local.

use super::{GenerationOutput, LlmGenerator, MistralLocalRuntime, RuntimeRequest};
use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::{LocalAiBackend, LocalModelDefinition};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;

pub struct MistralLocalProvider {
    runtime: Arc<MistralLocalRuntime>,
    model: LocalModelDefinition,
    path: PathBuf,
    backend: LocalAiBackend,
    temperature: f32,
}

impl MistralLocalProvider {
    #[must_use]
    pub fn new(
        runtime: Arc<MistralLocalRuntime>,
        model: LocalModelDefinition,
        path: PathBuf,
        backend: LocalAiBackend,
        temperature: f32,
    ) -> Self {
        Self {
            runtime,
            model,
            path,
            backend,
            temperature,
        }
    }
}

#[async_trait]
impl LlmGenerator for MistralLocalProvider {
    async fn generate(
        &self,
        prompt: &str,
        system: &str,
        json: bool,
    ) -> AppResult<GenerationOutput> {
        let runtime = Arc::clone(&self.runtime);
        let model_id = self.model.id;
        let expected_sha256 = self.model.sha256.clone();
        let path = self.path.clone();
        let backend = self.backend;
        let temperature = self.temperature;
        let prompt = prompt.to_owned();
        let system = system.to_owned();
        let output = tokio::task::spawn_blocking(move || {
            runtime.generate(RuntimeRequest {
                model_id,
                path: &path,
                expected_sha256: &expected_sha256,
                backend,
                system: &system,
                prompt: &prompt,
                temperature,
                json,
                max_output_tokens: None,
            })
        })
        .await
        .map_err(|error| {
            tracing::error!(%error, "tâche d'inférence locale interrompue");
            if error.is_panic() {
                AppError::Provider(
                    "L'IA locale a manqué de mémoire ou s'est interrompue. Réessayez avec un document plus court, ou choisissez un profil plus léger.".into(),
                )
            } else {
                AppError::Provider("L'inférence locale a été interrompue.".into())
            }
        })??;
        Ok(GenerationOutput {
            text: output.text,
            tokens: Some(output.prompt_tokens.saturating_add(output.generated_tokens)),
        })
    }

    async fn test(&self) -> AppResult<()> {
        self.generate(
            "Réponds uniquement par le mot prêt.",
            "Tu vérifies le bon fonctionnement du moteur local sans utiliser de donnée utilisateur.",
            false,
        )
        .await
        .map(|_| ())
    }

    async fn list_models(&self) -> AppResult<Vec<String>> {
        Ok(vec![self.model.display_name.clone()])
    }
}
