//! Accès aux fournisseurs, paramètres historiques et fichiers PDF.

mod config_repository;
mod pdf;
mod provider;

pub use config_repository::charger_config;
pub use pdf::extraire_pdf;
pub use provider::{construire_provider, GenerateurLlm};
