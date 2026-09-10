//! Client HTTP vers l'instance Ollama privée Candilog.

use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::{
    ManagedDownloadKind, ManagedModelId, ManagedOllamaDownloadProgress, ManagedRuntimeState,
};
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub struct ManagedOllamaApi {
    client: Client,
    base_url: String,
}

impl ManagedOllamaApi {
    pub fn new(base_url: String) -> AppResult<Self> {
        Ok(Self {
            client: Client::builder()
                .build()
                .map_err(|error| AppError::Provider(error.to_string()))?,
            base_url,
        })
    }

    pub async fn list_models(&self) -> AppResult<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(transport_error)?;
        let body: TagsResponse = response.json().await.map_err(transport_error)?;
        Ok(body.models.into_iter().map(|model| model.name).collect())
    }

    pub async fn pull_model(
        &self,
        tag: &str,
        model_id: ManagedModelId,
        cancel: &CancellationToken,
        on_progress: impl Fn(ManagedOllamaDownloadProgress),
    ) -> AppResult<()> {
        let response = self
            .client
            .post(format!("{}/api/pull", self.base_url))
            .json(&serde_json::json!({ "name": tag, "stream": true }))
            .send()
            .await
            .map_err(transport_error)?;
        if !response.status().is_success() {
            return Err(AppError::Provider(format!(
                "Téléchargement du modèle refusé ({})",
                response.status()
            )));
        }
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            if cancel.is_cancelled() {
                return Err(AppError::Cancelled);
            }
            let chunk = chunk.map_err(transport_error)?;
            if let Ok(event) = serde_json::from_slice::<PullEvent>(&chunk) {
                let downloaded = event.completed.unwrap_or(0);
                let total = event.total.unwrap_or(0);
                on_progress(ManagedOllamaDownloadProgress {
                    kind: ManagedDownloadKind::Model,
                    model_id: Some(model_id),
                    state: ManagedRuntimeState::Downloading,
                    downloaded_bytes: downloaded,
                    total_bytes: total,
                    progress: percent(downloaded, total),
                    label: format!("Téléchargement du modèle {tag}"),
                });
            }
        }
        Ok(())
    }

    pub async fn delete_model(&self, tag: &str) -> AppResult<()> {
        let response = self
            .client
            .delete(format!("{}/api/delete", self.base_url))
            .json(&serde_json::json!({ "name": tag }))
            .send()
            .await
            .map_err(transport_error)?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::Provider(
                "La suppression du modèle a échoué.".into(),
            ))
        }
    }

    pub fn models_disk_usage(models_dir: &Path) -> u64 {
        dir_size(models_dir)
    }
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0_u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += dir_size(&path);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn percent(downloaded: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    u8::try_from(downloaded.saturating_mul(100) / total)
        .unwrap_or(100)
        .min(100)
}

fn transport_error(error: reqwest::Error) -> AppError {
    AppError::Provider(format!("Le moteur local ne répond pas : {error}"))
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}

#[derive(Debug, Deserialize)]
struct TagModel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct PullEvent {
    completed: Option<u64>,
    total: Option<u64>,
}
