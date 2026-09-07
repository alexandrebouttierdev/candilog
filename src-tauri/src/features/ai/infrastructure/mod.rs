//! Accès aux fournisseurs, paramètres historiques et fichiers PDF.

mod config_repository;
mod local_downloader;
mod local_hardware;
mod local_provider;
mod local_runtime;
mod pdf;
mod provider;
mod system_resources;

pub use config_repository::load_config;
pub use local_downloader::{ModelDownload, ModelDownloader};
pub use local_hardware::detect_local_ai_hardware;
pub use local_provider::MistralLocalProvider;
pub use local_runtime::{
    MistralLocalRuntime, RuntimeGeneration, RuntimeRequest, LOCAL_AI_IDLE_UNLOAD,
};
pub use pdf::extract_pdf;
pub use provider::{build_provider, GenerationOutput, LlmGenerator};
pub use system_resources::{cache_gpu_vram, take_system_resource_snapshot};
