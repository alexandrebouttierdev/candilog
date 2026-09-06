//! Contrats du fournisseur Mistral Local et registre immuable des modèles.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const LOCAL_AI_CONTEXT_SIZE: u32 = 8_192;
pub const LOCAL_AI_EXTENDED_CONTEXT_SIZE: u32 = 16_384;
pub const LOCAL_AI_RUNTIME: &str = "llama.cpp";
pub const LOCAL_AI_QUANTIZATION: &str = "Q4_K_M";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum LocalModelId {
    Ministral3Light,
    Ministral3Balanced,
    Ministral3Quality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum LocalModelProfile {
    Light,
    Balanced,
    Quality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ModelCompatibility {
    Optimal,
    Supported,
    NotRecommended,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum LocalAiBackend {
    Metal,
    Cuda,
    Vulkan,
    Cpu,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalModelDefinition {
    pub id: LocalModelId,
    pub profile: LocalModelProfile,
    pub display_name: String,
    pub repository: String,
    pub filename: String,
    pub local_filename: String,
    pub revision: String,
    pub sha256: String,
    #[ts(type = "number")]
    pub download_size_bytes: u64,
    #[ts(type = "number")]
    pub estimated_ram_mb: u64,
    #[ts(type = "number")]
    pub recommended_ram_mb: u64,
    #[ts(type = "number")]
    pub recommended_vram_mb: Option<u64>,
    pub context_size: u32,
    pub quantization: String,
    pub runtime: String,
}

impl LocalModelDefinition {
    #[must_use]
    pub fn download_url(&self) -> String {
        format!(
            "https://huggingface.co/{}/resolve/{}/{}",
            self.repository, self.revision, self.filename
        )
    }
}

/// Source de vérité unique des trois artefacts téléchargeables.
pub struct ModelRegistry;

impl ModelRegistry {
    #[must_use]
    pub fn all() -> Vec<LocalModelDefinition> {
        vec![
            LocalModelDefinition {
                id: LocalModelId::Ministral3Light,
                profile: LocalModelProfile::Light,
                display_name: "Ministral 3 3B Instruct".into(),
                repository: "mistralai/Ministral-3-3B-Instruct-2512-GGUF".into(),
                filename: "Ministral-3-3B-Instruct-2512-Q4_K_M.gguf".into(),
                local_filename: "ministral-3b-q4_k_m.gguf".into(),
                revision: "eb599d408350ea2bb60452cb86be7c7b2fc28227".into(),
                sha256: "9ed150d4367e68df0ac8e1540f6ddc65b42d0ee26378329d1ecbca60f93fc5f8".into(),
                download_size_bytes: 2_147_023_008,
                estimated_ram_mb: 3_200,
                recommended_ram_mb: 8_192,
                recommended_vram_mb: Some(4_096),
                context_size: LOCAL_AI_CONTEXT_SIZE,
                quantization: LOCAL_AI_QUANTIZATION.into(),
                runtime: LOCAL_AI_RUNTIME.into(),
            },
            LocalModelDefinition {
                id: LocalModelId::Ministral3Balanced,
                profile: LocalModelProfile::Balanced,
                display_name: "Ministral 3 8B Instruct".into(),
                repository: "mistralai/Ministral-3-8B-Instruct-2512-GGUF".into(),
                filename: "Ministral-3-8B-Instruct-2512-Q4_K_M.gguf".into(),
                local_filename: "ministral-8b-q4_k_m.gguf".into(),
                revision: "0102285ad796bd99af90f58de616092e5630e970".into(),
                sha256: "33e7a72cf5e6e2cfc2f2847075acc013d68bba023e35310cef86b5cf8fdca761".into(),
                download_size_bytes: 5_198_911_904,
                estimated_ram_mb: 7_000,
                recommended_ram_mb: 16_384,
                recommended_vram_mb: Some(8_192),
                context_size: LOCAL_AI_CONTEXT_SIZE,
                quantization: LOCAL_AI_QUANTIZATION.into(),
                runtime: LOCAL_AI_RUNTIME.into(),
            },
            LocalModelDefinition {
                id: LocalModelId::Ministral3Quality,
                profile: LocalModelProfile::Quality,
                display_name: "Ministral 3 14B Instruct".into(),
                repository: "mistralai/Ministral-3-14B-Instruct-2512-GGUF".into(),
                filename: "Ministral-3-14B-Instruct-2512-Q4_K_M.gguf".into(),
                local_filename: "ministral-14b-q4_k_m.gguf".into(),
                revision: "74fac473c43357d7fb2671713608183cc72496d0".into(),
                sha256: "824e0f3373e69b84f2cae46fdcb9bd1ebc6ab3bfc7acc125d818b7b8178cc613".into(),
                download_size_bytes: 8_239_593_024,
                estimated_ram_mb: 11_000,
                recommended_ram_mb: 24_576,
                recommended_vram_mb: Some(12_288),
                context_size: LOCAL_AI_CONTEXT_SIZE,
                quantization: LOCAL_AI_QUANTIZATION.into(),
                runtime: LOCAL_AI_RUNTIME.into(),
            },
        ]
    }

    #[must_use]
    pub fn get(id: LocalModelId) -> Option<LocalModelDefinition> {
        Self::all().into_iter().find(|model| model.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalGpuInfo {
    pub name: String,
    pub vendor: String,
    #[ts(type = "number")]
    pub total_vram_mb: Option<u64>,
    #[ts(type = "number")]
    pub available_vram_mb: Option<u64>,
    pub backend: Option<LocalAiBackend>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalAiHardware {
    pub os: String,
    pub architecture: String,
    pub cpu: String,
    pub cpu_model: String,
    pub logical_cores: u32,
    pub physical_cores: Option<u32>,
    #[ts(type = "number")]
    pub total_ram_mb: u64,
    #[ts(type = "number")]
    pub available_ram_mb: u64,
    pub gpus: Vec<LocalGpuInfo>,
    pub apple_silicon: bool,
    pub soc_model: Option<String>,
    #[ts(type = "number")]
    pub unified_memory_mb: Option<u64>,
    pub metal: bool,
    pub cuda: bool,
    pub vulkan: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalModelEvaluation {
    pub model: LocalModelDefinition,
    pub compatibility: ModelCompatibility,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalAiRecommendation {
    pub hardware: LocalAiHardware,
    pub selected_model: Option<LocalModelDefinition>,
    pub backend: LocalAiBackend,
    pub evaluations: Vec<LocalModelEvaluation>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum LocalAiInstallationStatus {
    #[default]
    NotInstalled,
    Downloading,
    Verifying,
    Installing,
    Benchmarking,
    Installed,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum LocalAiState {
    NotConfigured,
    DetectingHardware,
    RecommendationReady,
    Downloading,
    Verifying,
    Installing,
    Benchmarking,
    Ready,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum BenchmarkRating {
    Excellent,
    Good,
    Acceptable,
    TooSlow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalAiBenchmark {
    #[ts(type = "number")]
    pub load_time_ms: u64,
    pub tokens_per_second: f32,
    #[ts(type = "number")]
    pub memory_used_mb: Option<u64>,
    pub generated_tokens: u32,
    pub rating: BenchmarkRating,
    pub measured_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct InstalledLocalModel {
    pub model_id: LocalModelId,
    pub profile: LocalModelProfile,
    pub revision: String,
    pub checksum: String,
    pub model_path: String,
    pub backend: LocalAiBackend,
    pub benchmark: Option<LocalAiBenchmark>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalAiSettings {
    #[serde(default)]
    pub selected_profile: Option<LocalModelProfile>,
    #[serde(default)]
    pub active_model_id: Option<LocalModelId>,
    #[serde(default)]
    pub installation_status: LocalAiInstallationStatus,
    #[serde(default)]
    pub installed_models: Vec<InstalledLocalModel>,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalAiStatus {
    pub state: LocalAiState,
    pub active_model: Option<LocalModelDefinition>,
    pub installed_models: Vec<LocalModelDefinition>,
    pub backend: Option<LocalAiBackend>,
    pub benchmark: Option<LocalAiBenchmark>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct InstallLocalAiRequest {
    pub model_id: LocalModelId,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalAiDownloadProgress {
    pub model_id: LocalModelId,
    pub state: LocalAiState,
    #[ts(type = "number")]
    pub downloaded_bytes: u64,
    #[ts(type = "number")]
    pub total_bytes: u64,
    #[ts(type = "number")]
    pub bytes_per_second: u64,
    pub progress: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalAiDownloadCompleted {
    pub model_id: LocalModelId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalAiDownloadError {
    pub model_id: LocalModelId,
    pub code: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LocalAiError {
    #[error("détection matérielle impossible")]
    HardwareDetectionFailed,
    #[error("matériel non pris en charge")]
    UnsupportedHardware,
    #[error("mémoire insuffisante")]
    InsufficientMemory,
    #[error("espace disque insuffisant")]
    InsufficientDiskSpace,
    #[error("téléchargement impossible")]
    DownloadFailed,
    #[error("téléchargement annulé")]
    DownloadCancelled,
    #[error("empreinte du modèle invalide")]
    InvalidChecksum,
    #[error("modèle introuvable")]
    ModelNotFound,
    #[error("chargement du modèle impossible")]
    ModelLoadFailed,
    #[error("runtime local indisponible")]
    RuntimeUnavailable,
    #[error("inférence locale impossible")]
    InferenceFailed,
    #[error("benchmark impossible")]
    BenchmarkFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_is_complete_and_consistent() {
        let models = ModelRegistry::all();
        assert_eq!(models.len(), 3);
        let mut ids = HashSet::new();
        for model in models {
            assert!(ids.insert(model.id));
            assert_eq!(model.repository.split('/').count(), 2);
            assert!(!model.filename.is_empty());
            assert!(model.local_filename.ends_with(".gguf"));
            assert_eq!(model.revision.len(), 40);
            assert_eq!(model.sha256.len(), 64);
            assert!(model
                .sha256
                .chars()
                .all(|character| character.is_ascii_hexdigit()));
            assert!(model.download_size_bytes > 0);
            assert!(model.estimated_ram_mb > model.download_size_bytes / 1_048_576);
            assert!(model.recommended_ram_mb >= model.estimated_ram_mb);
            assert!(model.recommended_vram_mb.is_some_and(|value| value > 0));
            assert_eq!(model.context_size, 8_192);
            assert_eq!(model.quantization, "Q4_K_M");
        }
        assert_eq!(
            ModelRegistry::get(LocalModelId::Ministral3Light).map(|m| m.profile),
            Some(LocalModelProfile::Light)
        );
        assert_eq!(
            ModelRegistry::get(LocalModelId::Ministral3Balanced).map(|m| m.profile),
            Some(LocalModelProfile::Balanced)
        );
        assert_eq!(
            ModelRegistry::get(LocalModelId::Ministral3Quality).map(|m| m.profile),
            Some(LocalModelProfile::Quality)
        );
    }

    #[test]
    fn registry_locks_the_three_verified_official_artifacts() {
        let models = ModelRegistry::all();
        let expected = [
            (
                "mistralai/Ministral-3-3B-Instruct-2512-GGUF",
                "Ministral-3-3B-Instruct-2512-Q4_K_M.gguf",
                "eb599d408350ea2bb60452cb86be7c7b2fc28227",
                "9ed150d4367e68df0ac8e1540f6ddc65b42d0ee26378329d1ecbca60f93fc5f8",
                2_147_023_008,
            ),
            (
                "mistralai/Ministral-3-8B-Instruct-2512-GGUF",
                "Ministral-3-8B-Instruct-2512-Q4_K_M.gguf",
                "0102285ad796bd99af90f58de616092e5630e970",
                "33e7a72cf5e6e2cfc2f2847075acc013d68bba023e35310cef86b5cf8fdca761",
                5_198_911_904,
            ),
            (
                "mistralai/Ministral-3-14B-Instruct-2512-GGUF",
                "Ministral-3-14B-Instruct-2512-Q4_K_M.gguf",
                "74fac473c43357d7fb2671713608183cc72496d0",
                "824e0f3373e69b84f2cae46fdcb9bd1ebc6ab3bfc7acc125d818b7b8178cc613",
                8_239_593_024,
            ),
        ];
        for (model, (repository, filename, revision, sha256, size)) in models.iter().zip(expected) {
            assert_eq!(model.repository, repository);
            assert_eq!(model.filename, filename);
            assert_eq!(model.revision, revision);
            assert_eq!(model.sha256, sha256);
            assert_eq!(model.download_size_bytes, size);
        }
    }
}
