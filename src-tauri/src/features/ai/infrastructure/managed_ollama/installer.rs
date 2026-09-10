//! Téléchargement et installation atomique du runtime Ollama géré.

use super::manifest::RuntimeArtifact;
use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::{
    ManagedDownloadKind, ManagedOllamaDownloadProgress, ManagedRuntimeState,
};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tar::Archive;
use tokio_util::sync::CancellationToken;

pub struct RuntimeInstaller {
    client: reqwest::Client,
}

impl RuntimeInstaller {
    pub fn new() -> AppResult<Self> {
        Ok(Self {
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()
                .map_err(|error| {
                    AppError::Provider(format!(
                        "Le client de téléchargement n'a pas pu être initialisé : {error}"
                    ))
                })?,
        })
    }

    pub async fn install(
        &self,
        artifact: &RuntimeArtifact,
        runtime_version_dir: &Path,
        downloads_dir: &Path,
        cancel: &CancellationToken,
        on_progress: &impl Fn(ManagedOllamaDownloadProgress),
    ) -> AppResult<PathBuf> {
        fs::create_dir_all(downloads_dir).map_err(map_io)?;
        fs::create_dir_all(runtime_version_dir).map_err(map_io)?;

        let archive_path = downloads_dir.join(artifact.archive_name);
        let part_path = downloads_dir.join(format!("{}.part", artifact.archive_name));
        let temp_dir = downloads_dir.join(format!("extract-{}", uuid::Uuid::new_v4()));
        let executable = runtime_version_dir.join("ollama");

        self.download_with_checksum(
            artifact,
            &part_path,
            &archive_path,
            cancel,
            |downloaded, total| {
                on_progress(ManagedOllamaDownloadProgress {
                    kind: ManagedDownloadKind::Runtime,
                    model_id: None,
                    state: ManagedRuntimeState::Downloading,
                    downloaded_bytes: downloaded,
                    total_bytes: total,
                    progress: percent(downloaded, total),
                    label: "Téléchargement du moteur".into(),
                });
            },
        )
        .await?;

        if cancel.is_cancelled() {
            return Err(AppError::Cancelled);
        }

        on_progress(ManagedOllamaDownloadProgress {
            kind: ManagedDownloadKind::Runtime,
            model_id: None,
            state: ManagedRuntimeState::Installing,
            downloaded_bytes: 0,
            total_bytes: 0,
            progress: 0,
            label: "Extraction du moteur…".into(),
        });

        let archive_for_extract = archive_path.clone();
        let temp_for_extract = temp_dir.clone();
        let executable_for_install = executable.clone();
        let executable_in_archive = artifact.executable.to_owned();
        let cancel_wait = cancel.clone();
        let extract_result = tokio::select! {
            result = tokio::task::spawn_blocking(move || {
                extract_runtime_archive(
                    &archive_for_extract,
                    &temp_for_extract,
                    &executable_for_install,
                    &executable_in_archive,
                )
            }) => result.map_err(|error| {
                tracing::error!(%error, "tâche d'extraction Ollama interrompue");
                AppError::Provider("L'extraction du moteur local a été interrompue.".into())
            })?,
            () = cancel_wait.cancelled() => {
                let _ = fs::remove_dir_all(&temp_dir);
                return Err(AppError::Cancelled);
            }
        };

        if cancel.is_cancelled() {
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(AppError::Cancelled);
        }

        extract_result?;
        Ok(executable)
    }

    async fn download_with_checksum(
        &self,
        artifact: &RuntimeArtifact,
        part_path: &Path,
        final_path: &Path,
        cancel: &CancellationToken,
        on_progress: impl Fn(u64, u64),
    ) -> AppResult<()> {
        let response = self
            .client
            .get(artifact.url)
            .send()
            .await
            .map_err(|error| AppError::Provider(format!("Téléchargement impossible : {error}")))?;
        if !response.status().is_success() {
            return Err(AppError::Provider(format!(
                "Téléchargement refusé ({})",
                response.status()
            )));
        }
        let total = response.content_length().unwrap_or(0);
        let mut downloaded = 0_u64;
        let mut hasher = Sha256::new();
        let mut file = File::create(part_path).map_err(map_io)?;
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            if cancel.is_cancelled() {
                let _ = fs::remove_file(part_path);
                return Err(AppError::Cancelled);
            }
            let chunk = chunk.map_err(|error| {
                AppError::Provider(format!("Téléchargement interrompu : {error}"))
            })?;
            hasher.update(&chunk);
            file.write_all(&chunk).map_err(map_io)?;
            downloaded += chunk.len() as u64;
            on_progress(downloaded, total);
        }
        file.flush().map_err(map_io)?;
        let digest = format!("{:x}", hasher.finalize());
        if digest != artifact.sha256 {
            let _ = fs::remove_file(part_path);
            return Err(AppError::Provider(
                "L'empreinte du moteur téléchargé ne correspond pas à la version attendue.".into(),
            ));
        }
        fs::rename(part_path, final_path).map_err(map_io)?;
        Ok(())
    }
}

fn extract_runtime_archive(
    archive_path: &Path,
    temp_dir: &Path,
    executable: &Path,
    executable_in_archive: &str,
) -> AppResult<()> {
    fs::create_dir_all(temp_dir).map_err(map_io)?;
    extract_tgz(archive_path, temp_dir)?;
    let extracted = temp_dir.join(executable_in_archive);
    if !extracted.exists() {
        return Err(AppError::Provider(
            "L'archive Ollama ne contient pas l'exécutable attendu.".into(),
        ));
    }
    fs::rename(&extracted, executable).map_err(map_io)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(executable, fs::Permissions::from_mode(0o700)).map_err(map_io)?;
    }
    let _ = fs::remove_dir_all(temp_dir);
    Ok(())
}

fn extract_tgz(archive_path: &Path, destination: &Path) -> AppResult<()> {
    let file = File::open(archive_path).map_err(map_io)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(destination).map_err(|error| {
        AppError::Provider(format!("Extraction de l'archive impossible : {error}"))
    })?;
    Ok(())
}

fn percent(downloaded: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    u8::try_from(downloaded.saturating_mul(100) / total)
        .unwrap_or(100)
        .min(100)
}

fn map_io(error: std::io::Error) -> AppError {
    tracing::error!(%error, "opération disque IA locale impossible");
    AppError::Provider("Une opération disque de l'IA locale a échoué.".into())
}
