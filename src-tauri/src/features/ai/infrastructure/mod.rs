//! Accès aux fournisseurs, paramètres historiques et fichiers PDF.

mod config_repository;
mod local_downloader;
mod local_hardware;
mod local_provider;
mod local_runtime;
mod pdf;
mod provider;

pub use config_repository::load_config;
pub use local_downloader::{ModelDownload, ModelDownloader};
pub use local_hardware::detect_local_ai_hardware;
pub use local_provider::MistralLocalProvider;
pub use local_runtime::{MistralLocalRuntime, RuntimeGeneration, RuntimeRequest};
pub use pdf::extract_pdf;
pub use provider::{build_provider, GenerationOutput, LlmGenerator};
