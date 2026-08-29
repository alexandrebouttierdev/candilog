//! Contracts et modèles de l'assistance IA.

mod config;
mod models;
mod scoring;

pub use config::*;
pub use models::*;
pub use scoring::{ground_generated_resume, profile_score, score_resume_imported};
