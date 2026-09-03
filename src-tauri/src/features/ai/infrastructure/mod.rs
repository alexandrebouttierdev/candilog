//! Accès aux fournisseurs, paramètres historiques et fichiers PDF.

mod config_repository;
mod pdf;
mod provider;

pub use config_repository::load_config;
pub use pdf::extract_pdf;
pub use provider::{build_provider, GenerationOutput, LlmGenerator};
