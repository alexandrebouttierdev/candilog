//! Accès aux fournisseurs, paramètres historiques et fichiers PDF.

mod benchmark_pdf;
mod company_sources;
mod config_repository;
mod managed_ollama;
mod pdf;
mod provider;
mod system_resources;

pub use benchmark_pdf::benchmark_pdf_path;
pub use company_sources::production_sources;
pub use config_repository::{load_config, load_searxng_url, load_task_config};
pub use managed_ollama::{
    require_runtime_artifact, ManagedOllamaApi, ManagedOllamaProcess, RuntimeArchiveFormat,
    RuntimeInstaller,
};
pub use pdf::{extract_pdf, render_pdf_pages, try_extract_pdf_text, PdfPageImage};
pub use provider::{
    build_provider, GenerationOutput, HttpProviderProbe, LlmGenerator, VisionImage,
};
pub use system_resources::{cache_gpu_vram, take_system_resource_snapshot};
