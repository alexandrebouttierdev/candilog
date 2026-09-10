//! Contrats du runtime Ollama géré par Candilog et catalogue des modèles locaux recommandés.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const MANAGED_OLLAMA_PREFERRED_PORT: u16 = 11_435;
pub const MANAGED_OLLAMA_RUNTIME_VERSION: &str = "0.13.4";

/// Version du benchmark utilisateur `CV_BENCHMARK.pdf` — à incrémenter si le PDF, la ground
/// truth ou l'algorithme de scoring change de façon incompatible.
pub const USER_BENCHMARK_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ManagedModelId {
    Lfm25350m,
    Lfm25UltraLight,
    Ministral3Light,
    Ministral3Balanced,
    Ministral3Powerful,
    MistralSmallQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ManagedModelCategory {
    UltraLight,
    Light,
    Balanced,
    Powerful,
    MaxQuality,
}

/// Éditeur du modèle Ollama — pilote le logo affiché sur les cartes du catalogue local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ManagedModelPublisher {
    Liquid,
    Mistral,
}

impl ManagedModelPublisher {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Liquid => "Liquid",
            Self::Mistral => "Mistral",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum MachineFit {
    Recommended,
    Compatible,
    MayBeSlow,
    InsufficientMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ManagedRuntimeState {
    #[default]
    NotInstalled,
    Downloading,
    Installing,
    Starting,
    Ready,
    Stopping,
    Stopped,
    Updating,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ManagedModelDefinition {
    pub id: ManagedModelId,
    pub category: ManagedModelCategory,
    pub publisher: ManagedModelPublisher,
    pub publisher_label: String,
    pub display_name: String,
    pub description: String,
    /// Tag Ollama utilisé pour pull et inférence.
    pub ollama_tag: String,
    #[ts(type = "number")]
    pub approximate_download_bytes: u64,
    #[ts(type = "number")]
    pub recommended_ram_gb: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ManagedModelStatus {
    pub definition: ManagedModelDefinition,
    pub installed: bool,
    pub active: bool,
    pub machine_fit: MachineFit,
    pub recommended: bool,
    pub last_benchmark: Option<UserBenchmarkSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct UserBenchmarkSummary {
    pub score: u32,
    pub total_ms: u32,
    pub benchmark_version: u32,
    pub measured_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ManagedOllamaStatus {
    pub runtime_state: ManagedRuntimeState,
    pub runtime_version: Option<String>,
    pub port: Option<u16>,
    #[ts(type = "number")]
    pub models_disk_bytes: u64,
    pub active_model: Option<ManagedModelDefinition>,
    pub models: Vec<ManagedModelStatus>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ManagedOllamaDownloadProgress {
    pub kind: ManagedDownloadKind,
    pub model_id: Option<ManagedModelId>,
    pub state: ManagedRuntimeState,
    #[ts(type = "number")]
    pub downloaded_bytes: u64,
    #[ts(type = "number")]
    pub total_bytes: u64,
    pub progress: u8,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ManagedDownloadKind {
    Runtime,
    Model,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ManagedOllamaSettings {
    #[serde(default)]
    pub active_model_id: Option<ManagedModelId>,
    #[serde(default)]
    pub installed_model_tags: Vec<String>,
    #[serde(default)]
    pub runtime_state: ManagedRuntimeState,
    #[serde(default)]
    pub runtime_port: Option<u16>,
    #[serde(default)]
    pub runtime_version: Option<String>,
    #[serde(default)]
    pub benchmark_history: Vec<StoredBenchmarkResult>,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct StoredBenchmarkResult {
    pub provider: String,
    pub model: String,
    pub benchmark_version: u32,
    pub score: u32,
    pub total_ms: u32,
    pub measured_at: String,
}

/// Catégories de score utilisateur pour l'affichage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum BenchmarkQualityLabel {
    Weak,
    Average,
    Fair,
    Good,
    VeryGood,
    Excellent,
}

#[must_use]
pub fn benchmark_quality_label(score: u32) -> BenchmarkQualityLabel {
    match score {
        0..=49 => BenchmarkQualityLabel::Weak,
        50..=64 => BenchmarkQualityLabel::Average,
        65..=74 => BenchmarkQualityLabel::Fair,
        75..=84 => BenchmarkQualityLabel::Good,
        85..=94 => BenchmarkQualityLabel::VeryGood,
        _ => BenchmarkQualityLabel::Excellent,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct UserBenchmarkCategoryScore {
    pub label: String,
    pub score: u32,
    pub max_score: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct UserBenchmarkMetrics {
    pub total_ms: u32,
    pub pdf_extract_ms: u32,
    pub preprocess_ms: u32,
    pub llm_ms: u32,
    pub parse_ms: u32,
    pub llm_calls: u32,
    pub tokens_input: Option<u32>,
    pub tokens_output: Option<u32>,
    pub tokens_per_second: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct UserBenchmarkResult {
    pub score: u32,
    pub quality: BenchmarkQualityLabel,
    pub metrics: UserBenchmarkMetrics,
    pub categories: Vec<UserBenchmarkCategoryScore>,
    pub hallucination_count: u32,
    pub benchmark_version: u32,
    pub provider_label: String,
    pub model_label: String,
    pub remote_warning: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct InstallManagedModelRequest {
    pub model_id: ManagedModelId,
}

pub struct ManagedModelRegistry;

impl ManagedModelRegistry {
    #[must_use]
    pub fn all() -> Vec<ManagedModelDefinition> {
        vec![
            ManagedModelDefinition {
                id: ManagedModelId::Lfm25350m,
                category: ManagedModelCategory::UltraLight,
                publisher: ManagedModelPublisher::Liquid,
                publisher_label: ManagedModelPublisher::Liquid.label().into(),
                display_name: "LFM2.5 350M".into(),
                description:
                    "Le plus léger du catalogue. Idéal pour tester l'IA locale sur une machine modeste."
                        .into(),
                ollama_tag: "maternion/lfm2.5:350m".into(),
                approximate_download_bytes: 350_000_000,
                recommended_ram_gb: 2,
            },
            ManagedModelDefinition {
                id: ManagedModelId::Lfm25UltraLight,
                category: ManagedModelCategory::UltraLight,
                publisher: ManagedModelPublisher::Liquid,
                publisher_label: ManagedModelPublisher::Liquid.label().into(),
                display_name: "LFM2.5 1.2B".into(),
                description:
                    "Rapide et peu gourmand en mémoire. Recommandé pour les ordinateurs modestes."
                        .into(),
                ollama_tag: "lfm2.5:1.2b".into(),
                approximate_download_bytes: 1_100_000_000,
                recommended_ram_gb: 4,
            },
            ManagedModelDefinition {
                id: ManagedModelId::Ministral3Light,
                category: ManagedModelCategory::Light,
                publisher: ManagedModelPublisher::Mistral,
                publisher_label: ManagedModelPublisher::Mistral.label().into(),
                display_name: "Ministral 3 3B".into(),
                description:
                    "Plus précis tout en restant adapté aux machines disposant de peu de mémoire."
                        .into(),
                ollama_tag: "ministral-3:3b".into(),
                approximate_download_bytes: 2_100_000_000,
                recommended_ram_gb: 8,
            },
            ManagedModelDefinition {
                id: ManagedModelId::Ministral3Balanced,
                category: ManagedModelCategory::Balanced,
                publisher: ManagedModelPublisher::Mistral,
                publisher_label: ManagedModelPublisher::Mistral.label().into(),
                display_name: "Ministral 3 8B".into(),
                description:
                    "Bon compromis entre qualité, vitesse et consommation mémoire.".into(),
                ollama_tag: "ministral-3:8b".into(),
                approximate_download_bytes: 5_200_000_000,
                recommended_ram_gb: 16,
            },
            ManagedModelDefinition {
                id: ManagedModelId::Ministral3Powerful,
                category: ManagedModelCategory::Powerful,
                publisher: ManagedModelPublisher::Mistral,
                publisher_label: ManagedModelPublisher::Mistral.label().into(),
                display_name: "Ministral 3 14B".into(),
                description:
                    "Meilleure qualité d'analyse pour les machines disposant de davantage de mémoire."
                        .into(),
                ollama_tag: "ministral-3:14b".into(),
                approximate_download_bytes: 8_200_000_000,
                recommended_ram_gb: 24,
            },
            ManagedModelDefinition {
                id: ManagedModelId::MistralSmallQuality,
                category: ManagedModelCategory::MaxQuality,
                publisher: ManagedModelPublisher::Mistral,
                publisher_label: ManagedModelPublisher::Mistral.label().into(),
                display_name: "Mistral Small 3.2 24B".into(),
                description:
                    "Modèle local plus exigeant, destiné aux machines puissantes et aux utilisateurs privilégiant la qualité."
                        .into(),
                ollama_tag: "mistral-small3.2:24b".into(),
                approximate_download_bytes: 15_200_000_000,
                recommended_ram_gb: 32,
            },
        ]
    }

    #[must_use]
    pub fn get(id: ManagedModelId) -> Option<ManagedModelDefinition> {
        Self::all().into_iter().find(|model| model.id == id)
    }

    #[must_use]
    pub fn by_tag(tag: &str) -> Option<ManagedModelDefinition> {
        let normalized = tag.trim();
        Self::all()
            .into_iter()
            .find(|model| model.ollama_tag == normalized)
    }
}

#[must_use]
pub fn evaluate_machine_fit(
    model: &ManagedModelDefinition,
    total_ram_gb: u32,
) -> (MachineFit, bool) {
    let recommended = total_ram_gb >= model.recommended_ram_gb;
    let fit = if total_ram_gb >= model.recommended_ram_gb {
        MachineFit::Recommended
    } else if total_ram_gb >= model.recommended_ram_gb.saturating_sub(2).max(4) {
        MachineFit::Compatible
    } else if total_ram_gb >= model.recommended_ram_gb / 2 {
        MachineFit::MayBeSlow
    } else {
        MachineFit::InsufficientMemory
    };
    (fit, recommended)
}
