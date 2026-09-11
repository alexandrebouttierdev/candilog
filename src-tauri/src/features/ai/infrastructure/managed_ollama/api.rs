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
        on_progress: &impl Fn(ManagedOllamaDownloadProgress),
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
        let mut buffer = String::new();
        while let Some(chunk) = stream.next().await {
            if cancel.is_cancelled() {
                return Err(AppError::Cancelled);
            }
            let chunk = chunk.map_err(transport_error)?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            for event in drain_pull_events(&mut buffer) {
                if let Some(progress) = pull_event_progress(&event, tag, model_id) {
                    on_progress(progress);
                }
            }
        }
        for event in drain_pull_events(&mut buffer) {
            if let Some(progress) = pull_event_progress(&event, tag, model_id) {
                on_progress(progress);
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
    status: Option<String>,
    completed: Option<u64>,
    total: Option<u64>,
}

fn drain_pull_events(buffer: &mut String) -> Vec<PullEvent> {
    let mut events = Vec::new();
    while let Some(newline) = buffer.find('\n') {
        let line = buffer[..newline].trim().to_owned();
        buffer.drain(..newline + 1);
        if line.is_empty() {
            continue;
        }
        if let Ok(event) = serde_json::from_str::<PullEvent>(&line) {
            events.push(event);
        }
    }
    events
}

fn pull_status_label(status: &str) -> String {
    match status {
        "pulling manifest" => "Récupération du manifeste…".into(),
        "verifying sha256 digest" => "Vérification de l'intégrité…".into(),
        "writing manifest" => "Enregistrement du manifeste…".into(),
        "removing any unused layers" => "Nettoyage des fichiers inutilisés…".into(),
        "success" => "Téléchargement terminé".into(),
        other if other.starts_with("downloading") => "Téléchargement en cours…".into(),
        other if other.starts_with("pulling") => "Récupération en cours…".into(),
        _ => "Téléchargement en cours…".into(),
    }
}

fn pull_event_progress(
    event: &PullEvent,
    _tag: &str,
    model_id: ManagedModelId,
) -> Option<ManagedOllamaDownloadProgress> {
    let label = event
        .status
        .as_deref()
        .filter(|status| !status.is_empty())
        .map(pull_status_label)
        .unwrap_or_else(|| "Téléchargement en cours…".into());
    if let Some(total) = event.total.filter(|value| *value > 0) {
        let downloaded = event.completed.unwrap_or(0);
        return Some(ManagedOllamaDownloadProgress {
            kind: ManagedDownloadKind::Model,
            model_id: Some(model_id),
            state: ManagedRuntimeState::Downloading,
            downloaded_bytes: downloaded,
            total_bytes: total,
            progress: percent(downloaded, total),
            label,
        });
    }
    if event.status.is_some() {
        return Some(ManagedOllamaDownloadProgress {
            kind: ManagedDownloadKind::Model,
            model_id: Some(model_id),
            state: ManagedRuntimeState::Downloading,
            downloaded_bytes: 0,
            total_bytes: 0,
            progress: 0,
            label,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drain_pull_events_gere_les_lignes_fragmentees() {
        let mut buffer = String::from(
            "{\"status\":\"downloading\",\"completed\":10,\"total\":100}\n{\"status\":\"downloading\",\"completed\":",
        );
        let first = drain_pull_events(&mut buffer);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].completed, Some(10));

        buffer.push_str("50,\"total\":100}\n");
        let second = drain_pull_events(&mut buffer);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].completed, Some(50));
        assert!(buffer.is_empty());
    }

    #[test]
    fn pull_event_progress_expose_les_etapes_sans_octets() {
        let event = PullEvent {
            status: Some("pulling manifest".into()),
            completed: None,
            total: None,
        };
        let progress = pull_event_progress(
            &event,
            "lfm2.5:ultra-light",
            ManagedModelId::Lfm25UltraLight,
        )
        .expect("progression manifeste");
        assert_eq!(progress.progress, 0);
        assert_eq!(progress.label, "Récupération du manifeste…");
    }

    #[test]
    fn pull_status_label_traduit_les_etapes_connues() {
        assert_eq!(
            pull_status_label("pulling manifest"),
            "Récupération du manifeste…"
        );
        assert_eq!(
            pull_status_label("downloading sha256:abc"),
            "Téléchargement en cours…"
        );
        assert_eq!(
            pull_status_label("verifying sha256 digest"),
            "Vérification de l'intégrité…"
        );
    }

    #[test]
    fn pull_event_progress_accepte_total_sans_completed() {
        let event = PullEvent {
            status: Some("downloading".into()),
            completed: None,
            total: Some(1_000),
        };
        let progress = pull_event_progress(
            &event,
            "lfm2.5:ultra-light",
            ManagedModelId::Lfm25UltraLight,
        )
        .expect("progression téléchargement");
        assert_eq!(progress.downloaded_bytes, 0);
        assert_eq!(progress.total_bytes, 1_000);
    }
}
