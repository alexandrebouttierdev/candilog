//! Contracts et modèles de l'assistance IA.

mod config;
mod cover_letter;
mod models;
mod normalization;
mod scoring;
mod validation;

pub use config::*;
pub use cover_letter::*;
pub use models::*;
pub use normalization::search_key;
pub use scoring::{
    ground_extracted_listing, ground_generated_resume, ground_imported_resume, profile_score,
    score_resume_imported,
};
pub use validation::*;
