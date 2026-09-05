//! Contracts et modèles de l'assistance IA.

mod config;
mod cover_letter;
mod models;
mod normalization;
mod scoring;
mod validation;

pub use crate::core::utils::text::search_key;
pub use config::*;
pub use cover_letter::*;
pub use models::*;
pub use scoring::{
    ground_content_recommendations, ground_extracted_listing, ground_generated_resume,
    ground_imported_resume, profile_content_catalog, profile_score, score_resume_imported,
    ProfileContentCatalogEntry,
};
pub use validation::*;
