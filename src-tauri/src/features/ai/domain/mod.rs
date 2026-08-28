//! Contracts et modèles de l'assistance IA.

mod config;
mod models;
mod scoring;

pub use config::*;
pub use models::*;
pub use scoring::{score_resume_imported, profile_score};
