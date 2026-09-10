//! Orchestration du runtime Ollama privé et des modèles Candilog.

use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::{
    evaluate_machine_fit, InstallManagedModelRequest, ManagedModelDefinition, ManagedModelId,
    ManagedModelRegistry, ManagedModelStatus, ManagedOllamaDownloadProgress, ManagedOllamaStatus,
    ManagedRuntimeState, StoredBenchmarkResult, UserBenchmarkSummary,
};
use crate::features::ai::infrastructure::{
    require_runtime_artifact, ManagedOllamaApi, ManagedOllamaProcess, RuntimeInstaller,
};
use crate::features::settings::domain::SettingsRepository;
use crate::features::settings::infrastructure::SqliteSettingsRepository;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use tokio_util::sync::CancellationToken;

pub struct ManagedOllamaPaths {
    pub runtime_root: PathBuf,
    pub models_dir: PathBuf,
    pub downloads_dir: PathBuf,
}

pub struct ManagedOllamaService {
    pool: SqlitePool,
    paths: ManagedOllamaPaths,
    process: Arc<Mutex<Option<ManagedOllamaProcess>>>,
    download: Mutex<Option<(ManagedModelId, CancellationToken)>>,
}

impl ManagedOllamaService {
    pub fn new(pool: SqlitePool, paths: ManagedOllamaPaths) -> Self {
        Self {
            pool,
            paths,
            process: Arc::new(Mutex::new(None)),
            download: Mutex::new(None),
        }
    }

    pub fn status(&self) -> AppResult<ManagedOllamaStatus> {
        let settings = self.settings()?.managed_ollama;
        let total_ram_gb = (System::new_all().total_memory() / 1_073_741_824).max(1) as u32;
        let installed_tags = settings.installed_model_tags.clone();
        let models = ManagedModelRegistry::all()
            .into_iter()
            .map(|definition| {
                let installed = installed_tags
                    .iter()
                    .any(|tag| tag == &definition.ollama_tag);
                let (machine_fit, recommended) = evaluate_machine_fit(&definition, total_ram_gb);
                let last_benchmark = settings
                    .benchmark_history
                    .iter()
                    .find(|entry| entry.model == definition.ollama_tag)
                    .map(|entry| UserBenchmarkSummary {
                        score: entry.score,
                        total_ms: entry.total_ms,
                        benchmark_version: entry.benchmark_version,
                        measured_at: entry.measured_at.clone(),
                    });
                let model_id = definition.id;
                ManagedModelStatus {
                    definition,
                    installed,
                    active: settings.active_model_id == Some(model_id),
                    machine_fit,
                    recommended,
                    last_benchmark,
                }
            })
            .collect();
        let active_model = settings.active_model_id.and_then(ManagedModelRegistry::get);
        Ok(ManagedOllamaStatus {
            runtime_state: settings.runtime_state,
            runtime_version: settings.runtime_version.clone(),
            port: settings.runtime_port,
            models_disk_bytes: ManagedOllamaApi::models_disk_usage(&self.paths.models_dir),
            active_model,
            models,
            last_error: settings.last_error.clone(),
        })
    }

    pub async fn ensure_runtime_ready(&self) -> AppResult<String> {
        let artifact = require_runtime_artifact()?;
        let version_dir = self.paths.runtime_root.join(artifact.version);
        let executable = version_dir.join("ollama");
        if !executable.exists() {
            let installer = RuntimeInstaller::new()?;
            let cancel = CancellationToken::new();
            installer
                .install(
                    &artifact,
                    &version_dir,
                    &self.paths.downloads_dir,
                    &cancel,
                    |_| {},
                )
                .await?;
            self.persist_runtime_state(ManagedRuntimeState::Ready, Some(artifact.version), None)?;
        }
        let port = {
            let mut guard = self.process.lock().map_err(lock_err)?;
            if guard.is_none() {
                *guard = Some(ManagedOllamaProcess::new(
                    executable,
                    self.paths.models_dir.clone(),
                ));
            }
            guard
                .as_mut()
                .ok_or_else(|| AppError::Provider("Moteur local indisponible.".into()))?
                .ensure_running()?
        };
        self.persist_runtime_state(
            ManagedRuntimeState::Ready,
            Some(artifact.version),
            Some(port),
        )?;
        Ok(format!("http://127.0.0.1:{port}"))
    }

    pub async fn install_model(
        &self,
        request: InstallManagedModelRequest,
        on_progress: impl Fn(ManagedOllamaDownloadProgress),
    ) -> AppResult<()> {
        let definition = ManagedModelRegistry::get(request.model_id)
            .ok_or_else(|| AppError::Validation("Modèle local inconnu.".into()))?;
        if self.download.lock().map_err(lock_err)?.is_some() {
            return Err(AppError::Provider(
                "Un téléchargement est déjà en cours.".into(),
            ));
        }
        let cancel = CancellationToken::new();
        *self.download.lock().map_err(lock_err)? = Some((request.model_id, cancel.clone()));
        let base_url = self.ensure_runtime_ready().await?;
        let api = ManagedOllamaApi::new(base_url)?;
        let result = api
            .pull_model(
                &definition.ollama_tag,
                request.model_id,
                &cancel,
                on_progress,
            )
            .await;
        *self.download.lock().map_err(lock_err)? = None;
        result?;
        let mut settings = self.settings()?;
        if !settings
            .managed_ollama
            .installed_model_tags
            .iter()
            .any(|tag| tag == &definition.ollama_tag)
        {
            settings
                .managed_ollama
                .installed_model_tags
                .push(definition.ollama_tag.clone());
        }
        settings.managed_ollama.active_model_id = Some(request.model_id);
        settings.managed_ollama.last_error = None;
        self.save_settings(&settings)?;
        Ok(())
    }

    pub fn cancel_download(&self) -> AppResult<()> {
        if let Some((_, token)) = self.download.lock().map_err(lock_err)?.take() {
            token.cancel();
        }
        Ok(())
    }

    pub async fn remove_model(&self, model_id: ManagedModelId) -> AppResult<()> {
        let definition = ManagedModelRegistry::get(model_id)
            .ok_or_else(|| AppError::Validation("Modèle local inconnu.".into()))?;
        if let Ok(base_url) = self.ensure_runtime_ready().await {
            let api = ManagedOllamaApi::new(base_url)?;
            let _ = api.delete_model(&definition.ollama_tag).await;
        }
        let mut settings = self.settings()?;
        settings
            .managed_ollama
            .installed_model_tags
            .retain(|tag| tag != &definition.ollama_tag);
        if settings.managed_ollama.active_model_id == Some(model_id) {
            settings.managed_ollama.active_model_id = None;
        }
        self.save_settings(&settings)?;
        Ok(())
    }

    pub fn activate_model(&self, model_id: ManagedModelId) -> AppResult<ManagedModelDefinition> {
        let definition = ManagedModelRegistry::get(model_id)
            .ok_or_else(|| AppError::Validation("Modèle local inconnu.".into()))?;
        let mut settings = self.settings()?;
        if !settings
            .managed_ollama
            .installed_model_tags
            .iter()
            .any(|tag| tag == &definition.ollama_tag)
        {
            return Err(AppError::Provider(
                "Ce modèle n'est pas encore installé.".into(),
            ));
        }
        settings.managed_ollama.active_model_id = Some(model_id);
        settings.llm.provider = crate::features::ai::domain::ProviderKind::CandilogLocal;
        settings.llm.model = definition.ollama_tag.clone();
        self.save_settings(&settings)?;
        Ok(definition)
    }

    pub fn record_benchmark(&self, entry: StoredBenchmarkResult) -> AppResult<()> {
        let mut settings = self.settings()?;
        settings
            .managed_ollama
            .benchmark_history
            .retain(|item| item.model != entry.model);
        settings.managed_ollama.benchmark_history.push(entry);
        self.save_settings(&settings)
    }

    pub fn active_ollama_tag(&self) -> AppResult<Option<String>> {
        let settings = self.settings()?;
        Ok(settings
            .managed_ollama
            .active_model_id
            .and_then(ManagedModelRegistry::get)
            .map(|model| model.ollama_tag))
    }

    pub fn shutdown(&self) {
        if let Ok(mut guard) = self.process.lock() {
            if let Some(process) = guard.as_ref() {
                let _ = process.stop();
            }
            *guard = None;
        }
    }

    fn settings(&self) -> AppResult<crate::features::settings::domain::AppSettings> {
        SqliteSettingsRepository::new(self.pool.clone()).get()
    }

    fn save_settings(
        &self,
        settings: &crate::features::settings::domain::AppSettings,
    ) -> AppResult<()> {
        SqliteSettingsRepository::new(self.pool.clone())
            .upsert(settings)
            .map(|_| ())
    }

    fn persist_runtime_state(
        &self,
        state: ManagedRuntimeState,
        version: Option<&str>,
        port: Option<u16>,
    ) -> AppResult<()> {
        let mut settings = self.settings()?;
        settings.managed_ollama.runtime_state = state;
        if let Some(version) = version {
            settings.managed_ollama.runtime_version = Some(version.to_owned());
        }
        if let Some(port) = port {
            settings.managed_ollama.runtime_port = Some(port);
        }
        self.save_settings(&settings)
    }
}

fn lock_err<T>(_: std::sync::PoisonError<T>) -> AppError {
    AppError::Provider("État interne du moteur local corrompu.".into())
}
