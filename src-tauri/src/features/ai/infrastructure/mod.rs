//! Accès aux fournisseurs, paramètres historiques et fichiers PDF.

mod config_repository;
mod managed_ollama;
mod pdf;
mod provider;
mod system_resources;

pub use config_repository::load_config;
pub use managed_ollama::{
    require_runtime_artifact, ManagedOllamaApi, ManagedOllamaProcess, RuntimeInstaller,
};
pub use pdf::extract_pdf;
pub use provider::{build_provider, GenerationOutput, LlmGenerator};
pub use system_resources::{cache_gpu_vram, take_system_resource_snapshot};
