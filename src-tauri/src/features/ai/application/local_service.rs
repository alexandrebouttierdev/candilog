//! Orchestration de l'installation et du cycle de vie de Mistral Local.

use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::{
    BenchmarkRating, InstallLocalAiRequest, InstalledLocalModel, LocalAiBenchmark,
    LocalAiDownloadProgress, LocalAiInstallationStatus, LocalAiRecommendation, LocalAiState,
    LocalAiStatus, LocalModelDefinition, LocalModelId, ModelCompatibility, ModelRegistry,
    ProviderKind,
};
use crate::features::ai::infrastructure::{
    detect_local_ai_hardware, MistralLocalProvider, MistralLocalRuntime, ModelDownload,
    ModelDownloader, RuntimeRequest,
};
use crate::features::settings::domain::SettingsRepository;
use crate::features::settings::infrastructure::SqliteSettingsRepository;
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

pub struct LocalAiService {
    pool: SqlitePool,
    models_dir: PathBuf,
    runtime: Arc<MistralLocalRuntime>,
    downloader: ModelDownloader,
    download: Mutex<Option<(LocalModelId, CancellationToken)>>,
}

impl LocalAiService {
    pub fn new(pool: SqlitePool, models_dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&models_dir).map_err(|error| {
            tracing::error!(%error, "dossier des modèles locaux non créé");
            AppError::Provider("Le dossier de l'IA locale n'a pas pu être créé.".into())
        })?;
        Ok(Self {
            pool,
            models_dir,
            runtime: Arc::new(MistralLocalRuntime::new()),
            downloader: ModelDownloader::new()?,
            download: Mutex::new(None),
        })
    }

    #[must_use]
    pub fn detect_hardware(&self) -> crate::features::ai::domain::LocalAiHardware {
        detect_local_ai_hardware()
    }

    #[must_use]
    pub fn recommendation(&self) -> LocalAiRecommendation {
        let recommendation = super::LocalModelSelector::recommend(self.detect_hardware());
        tracing::info!(
            backend = ?recommendation.backend,
            model = ?recommendation.selected_model.as_ref().map(|model| model.id),
            "configuration Mistral Local évaluée"
        );
        recommendation
    }

    pub fn status(&self) -> AppResult<LocalAiStatus> {
        let settings = self.settings()?.local_ai;
        let installed_models = settings
            .installed_models
            .iter()
            .filter_map(|installed| {
                let definition = ModelRegistry::get(installed.model_id)?;
                self.valid_installed_path(installed, &definition)
                    .then_some(definition)
            })
            .collect();
        let active = settings
            .active_model_id
            .and_then(ModelRegistry::get)
            .filter(|definition| {
                settings.installed_models.iter().any(|installed| {
                    installed.model_id == definition.id
                        && self.valid_installed_path(installed, definition)
                })
            });
        let installed = active.as_ref().and_then(|definition| {
            settings
                .installed_models
                .iter()
                .find(|model| model.model_id == definition.id)
        });
        let state = match settings.installation_status {
            LocalAiInstallationStatus::Downloading => LocalAiState::Downloading,
            LocalAiInstallationStatus::Verifying => LocalAiState::Verifying,
            LocalAiInstallationStatus::Installing => LocalAiState::Installing,
            LocalAiInstallationStatus::Benchmarking => LocalAiState::Benchmarking,
            LocalAiInstallationStatus::Error => LocalAiState::Error,
            LocalAiInstallationStatus::Installed if active.is_some() => LocalAiState::Ready,
            LocalAiInstallationStatus::Installed | LocalAiInstallationStatus::NotInstalled => {
                LocalAiState::NotConfigured
            }
        };
        Ok(LocalAiStatus {
            state,
            active_model: active,
            installed_models,
            backend: installed.map(|model| model.backend),
            benchmark: installed.and_then(|model| model.benchmark.clone()),
            last_error: settings.last_error,
        })
    }

    pub async fn install(
        &self,
        request: InstallLocalAiRequest,
        progress: impl Fn(LocalAiDownloadProgress),
    ) -> AppResult<LocalAiStatus> {
        let model = ModelRegistry::get(request.model_id)
            .ok_or_else(|| AppError::Validation("Le profil local demandé n'existe pas.".into()))?;
        self.ensure_supported(&model)?;
        let cancellation = self.start_download(model.id)?;
        let result = self.install_inner(&model, &cancellation, &progress).await;
        self.finish_download(model.id);
        match result {
            Ok(status) => Ok(status),
            Err(error) => {
                self.persist_error(error.user_message())?;
                Err(error)
            }
        }
    }

    async fn install_inner(
        &self,
        model: &LocalModelDefinition,
        cancellation: &CancellationToken,
        progress: &impl Fn(LocalAiDownloadProgress),
    ) -> AppResult<LocalAiStatus> {
        self.set_installation_status(LocalAiInstallationStatus::Downloading)?;
        tracing::info!(model = ?model.id, size = model.download_size_bytes, "téléchargement Mistral Local démarré");
        let destination = self.model_path(model)?;
        let download_url = model.download_url();
        let path = self
            .downloader
            .download(
                ModelDownload {
                    model_id: model.id,
                    url: &download_url,
                    destination: &destination,
                    expected_size: model.download_size_bytes,
                    expected_sha256: &model.sha256,
                },
                cancellation,
                |event| {
                    if event.state == LocalAiState::Verifying {
                        let _ = self.set_installation_status(LocalAiInstallationStatus::Verifying);
                    }
                    progress(event);
                },
            )
            .await?;
        self.set_installation_status(LocalAiInstallationStatus::Installing)?;
        progress(LocalAiDownloadProgress {
            model_id: model.id,
            state: LocalAiState::Installing,
            downloaded_bytes: model.download_size_bytes,
            total_bytes: model.download_size_bytes,
            bytes_per_second: 0,
            progress: 100,
        });
        let backend = self.recommendation().backend;
        self.update_settings(|settings| {
            settings
                .local_ai
                .installed_models
                .retain(|entry| entry.model_id != model.id);
            settings
                .local_ai
                .installed_models
                .push(InstalledLocalModel {
                    model_id: model.id,
                    profile: model.profile,
                    revision: model.revision.clone(),
                    checksum: model.sha256.clone(),
                    model_path: path.to_string_lossy().into_owned(),
                    backend,
                    benchmark: None,
                });
            settings.local_ai.installation_status = LocalAiInstallationStatus::Benchmarking;
            settings.local_ai.last_error = None;
        })?;
        progress(LocalAiDownloadProgress {
            model_id: model.id,
            state: LocalAiState::Benchmarking,
            downloaded_bytes: model.download_size_bytes,
            total_bytes: model.download_size_bytes,
            bytes_per_second: 0,
            progress: 100,
        });
        let benchmark = self.run_benchmark(model.id, true).await?;
        self.update_settings(|settings| {
            if let Some(entry) = settings
                .local_ai
                .installed_models
                .iter_mut()
                .find(|entry| entry.model_id == model.id)
            {
                entry.benchmark = Some(benchmark.clone());
            }
            settings.local_ai.selected_profile = Some(model.profile);
            settings.local_ai.active_model_id = Some(model.id);
            settings.local_ai.installation_status = LocalAiInstallationStatus::Installed;
            settings.local_ai.last_error = None;
            settings.llm.provider = ProviderKind::MistralLocal;
            settings.llm.endpoint = None;
            settings.llm.model.clone_from(&model.display_name);
        })?;
        tracing::info!(model = ?model.id, "installation Mistral Local terminée");
        self.status()
    }

    pub fn cancel_download(&self) {
        if let Some((_, token)) = self
            .download
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            token.cancel();
        }
    }

    pub async fn benchmark(&self) -> AppResult<LocalAiBenchmark> {
        let id = self
            .settings()?
            .local_ai
            .active_model_id
            .ok_or_else(|| AppError::NotFound("modèle Mistral Local actif".into()))?;
        self.set_installation_status(LocalAiInstallationStatus::Benchmarking)?;
        let result = self.run_benchmark(id, true).await;
        match result {
            Ok(benchmark) => {
                self.update_settings(|settings| {
                    if let Some(entry) = settings
                        .local_ai
                        .installed_models
                        .iter_mut()
                        .find(|entry| entry.model_id == id)
                    {
                        entry.benchmark = Some(benchmark.clone());
                    }
                    settings.local_ai.installation_status = LocalAiInstallationStatus::Installed;
                    settings.local_ai.last_error = None;
                })?;
                Ok(benchmark)
            }
            Err(error) => {
                self.persist_error(error.user_message())?;
                Err(error)
            }
        }
    }

    pub async fn test_model(&self) -> AppResult<String> {
        let provider = self.provider(0.1)?;
        let output = crate::features::ai::infrastructure::LlmGenerator::generate(
            provider.as_ref(),
            "Réponds en une phrase courte confirmant que l'assistance locale est prête.",
            "Tu réponds en français professionnel. Ce test ne contient aucune donnée utilisateur.",
            false,
        )
        .await?;
        Ok(output.text)
    }

    pub async fn remove(&self, model_id: LocalModelId) -> AppResult<LocalAiStatus> {
        let model = ModelRegistry::get(model_id)
            .ok_or_else(|| AppError::Validation("Le profil local demandé n'existe pas.".into()))?;
        let path = self.model_path(&model)?;
        let part = path.with_file_name(format!("{}.part", model.local_filename));
        let active = self.settings()?.local_ai.active_model_id == Some(model_id);
        if active {
            self.runtime.unload();
        }
        remove_if_exists(&path).await?;
        remove_if_exists(&part).await?;
        self.update_settings(|settings| {
            settings
                .local_ai
                .installed_models
                .retain(|entry| entry.model_id != model_id);
            if settings.local_ai.active_model_id == Some(model_id) {
                settings.local_ai.active_model_id = None;
                settings.local_ai.selected_profile = None;
                settings.llm.model.clear();
            }
            settings.local_ai.installation_status = if settings.local_ai.active_model_id.is_some() {
                LocalAiInstallationStatus::Installed
            } else {
                LocalAiInstallationStatus::NotInstalled
            };
            settings.local_ai.last_error = None;
        })?;
        tracing::info!(model = ?model_id, "modèle Mistral Local supprimé");
        self.status()
    }

    pub fn provider(
        &self,
        temperature: f32,
    ) -> AppResult<Arc<dyn crate::features::ai::infrastructure::LlmGenerator>> {
        let settings = self.settings()?.local_ai;
        let id = settings.active_model_id.ok_or_else(|| {
            AppError::Provider("Installez l'IA locale avant de l'utiliser.".into())
        })?;
        let model = ModelRegistry::get(id).ok_or_else(|| {
            AppError::Provider("Le modèle local enregistré n'existe plus.".into())
        })?;
        let installed = settings
            .installed_models
            .iter()
            .find(|entry| entry.model_id == id)
            .ok_or_else(|| {
                AppError::Provider("Le modèle local enregistré est introuvable.".into())
            })?;
        if !self.valid_installed_path(installed, &model) {
            return Err(AppError::Provider(
                "Le modèle local est absent ou ne correspond plus à la version installée.".into(),
            ));
        }
        Ok(Arc::new(MistralLocalProvider::new(
            Arc::clone(&self.runtime),
            model,
            self.model_path_for_id(id)?,
            installed.backend,
            temperature,
        )))
    }

    pub fn shutdown(&self) {
        self.cancel_download();
        self.runtime.unload();
    }

    async fn run_benchmark(
        &self,
        id: LocalModelId,
        force_reload: bool,
    ) -> AppResult<LocalAiBenchmark> {
        let model = ModelRegistry::get(id)
            .ok_or_else(|| AppError::NotFound("modèle Mistral Local".into()))?;
        let settings = self.settings()?.local_ai;
        let installed = settings
            .installed_models
            .iter()
            .find(|entry| entry.model_id == id)
            .ok_or_else(|| AppError::NotFound("modèle Mistral Local installé".into()))?;
        let backend = installed.backend;
        let path = self.model_path(&model)?;
        if force_reload {
            self.runtime.unload();
        }
        let runtime = Arc::clone(&self.runtime);
        let expected_sha256 = model.sha256.clone();
        let before = process_memory_mb();
        let started = std::time::Instant::now();
        let output = tokio::task::spawn_blocking(move || {
            runtime.generate(RuntimeRequest {
                model_id: id,
                path: &path,
                expected_sha256: &expected_sha256,
                backend,
                system: "Tu exécutes un benchmark synthétique sans donnée utilisateur.",
                prompt: "Rédige quatre phrases très courtes en français sur l'organisation d'une recherche d'emploi.",
                temperature: 0.2,
                json: false,
                max_output_tokens: Some(64),
            })
        })
        .await
        .map_err(|error| AppError::Provider(error.to_string()))??;
        let elapsed_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        let generation_ms = elapsed_ms.saturating_sub(output.load_time_ms).max(1);
        let tokens_per_second = output.generated_tokens as f32 * 1_000.0 / generation_ms as f32;
        let rating = if tokens_per_second > 20.0 {
            BenchmarkRating::Excellent
        } else if tokens_per_second >= 10.0 {
            BenchmarkRating::Good
        } else if tokens_per_second >= 5.0 {
            BenchmarkRating::Acceptable
        } else {
            BenchmarkRating::TooSlow
        };
        let memory_used_mb =
            process_memory_mb().and_then(|after| before.map(|value| after.saturating_sub(value)));
        let benchmark = LocalAiBenchmark {
            load_time_ms: output.load_time_ms,
            tokens_per_second,
            memory_used_mb,
            generated_tokens: output.generated_tokens,
            rating,
            measured_at: Utc::now().to_rfc3339(),
        };
        tracing::info!(model = ?id, backend = ?output.backend, tokens_per_second, load_time_ms = output.load_time_ms, "benchmark Mistral Local terminé");
        Ok(benchmark)
    }

    fn ensure_supported(&self, model: &LocalModelDefinition) -> AppResult<()> {
        let recommendation = self.recommendation();
        let compatibility = recommendation
            .evaluations
            .iter()
            .find(|evaluation| evaluation.model.id == model.id)
            .map(|evaluation| evaluation.compatibility)
            .unwrap_or(ModelCompatibility::Unsupported);
        if matches!(
            compatibility,
            ModelCompatibility::Optimal | ModelCompatibility::Supported
        ) {
            Ok(())
        } else {
            Err(AppError::Validation(
                "Ce profil n'est pas recommandé sur cette machine. Réévaluez la configuration et choisissez le profil proposé."
                    .into(),
            ))
        }
    }

    fn start_download(&self, id: LocalModelId) -> AppResult<CancellationToken> {
        let mut current = self
            .download
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if current.is_some() {
            return Err(AppError::Validation(
                "Un téléchargement d'IA locale est déjà en cours.".into(),
            ));
        }
        let token = CancellationToken::new();
        *current = Some((id, token.clone()));
        Ok(token)
    }

    fn finish_download(&self, id: LocalModelId) {
        let mut current = self
            .download
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if current
            .as_ref()
            .is_some_and(|(current_id, _)| *current_id == id)
        {
            current.take();
        }
    }

    fn settings(&self) -> AppResult<crate::features::settings::domain::AppSettings> {
        SqliteSettingsRepository::new(self.pool.clone()).get()
    }

    fn update_settings(
        &self,
        update: impl FnOnce(&mut crate::features::settings::domain::AppSettings),
    ) -> AppResult<()> {
        let repository = SqliteSettingsRepository::new(self.pool.clone());
        let mut settings = repository.get()?;
        update(&mut settings);
        repository.upsert(&settings)?;
        Ok(())
    }

    fn set_installation_status(&self, status: LocalAiInstallationStatus) -> AppResult<()> {
        self.update_settings(|settings| {
            settings.local_ai.installation_status = status;
            settings.local_ai.last_error = None;
        })
    }

    fn persist_error(&self, message: String) -> AppResult<()> {
        self.update_settings(|settings| {
            settings.local_ai.installation_status = LocalAiInstallationStatus::Error;
            settings.local_ai.last_error = Some(message);
        })
    }

    fn model_path(&self, model: &LocalModelDefinition) -> AppResult<PathBuf> {
        if model.local_filename.contains('/')
            || model.local_filename.contains('\\')
            || !model.local_filename.ends_with(".gguf")
        {
            return Err(AppError::Validation(
                "Le nom du modèle local est invalide.".into(),
            ));
        }
        Ok(self.models_dir.join(&model.local_filename))
    }

    fn model_path_for_id(&self, id: LocalModelId) -> AppResult<PathBuf> {
        let model = ModelRegistry::get(id)
            .ok_or_else(|| AppError::NotFound("modèle Mistral Local".into()))?;
        self.model_path(&model)
    }

    fn valid_installed_path(
        &self,
        installed: &InstalledLocalModel,
        model: &LocalModelDefinition,
    ) -> bool {
        let Ok(expected) = self.model_path(model) else {
            return false;
        };
        Path::new(&installed.model_path) == expected
            && installed.revision == model.revision
            && installed.checksum.eq_ignore_ascii_case(&model.sha256)
            && std::fs::metadata(expected).is_ok_and(|metadata| {
                metadata.is_file() && metadata.len() == model.download_size_bytes
            })
    }
}

async fn remove_if_exists(path: &Path) -> AppResult<()> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            tracing::error!(%error, "modèle local non supprimé");
            Err(AppError::Provider(
                "Le modèle local n'a pas pu être supprimé.".into(),
            ))
        }
    }
}

fn process_memory_mb() -> Option<u64> {
    let pid = sysinfo::get_current_pid().ok()?;
    let system = sysinfo::System::new_all();
    system
        .process(pid)
        .map(|process| process.memory() / 1_048_576)
}
