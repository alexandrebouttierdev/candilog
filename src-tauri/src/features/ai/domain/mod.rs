//! Contrats et modèles de l'assistance IA.

mod config;
mod cover_letter;
mod local_ai;
mod models;
mod normalization;
mod profile_dates;
mod profile_grounding;
mod scoring;
mod system_resources;
mod validation;

pub use crate::core::utils::text::search_key;
pub use config::*;
pub use cover_letter::*;
pub use local_ai::*;
pub use models::*;
pub use profile_dates::normalize_profile_dates;
pub use profile_grounding::{completer_contacts_vides, completer_formations_manquantes, ground_imported_profile};
pub use scoring::{
    ground_content_recommendations, ground_extracted_listing, ground_generated_resume,
    ground_imported_resume, profile_content_catalog, profile_score, score_resume_imported,
    ProfileContentCatalogEntry,
};
pub use system_resources::*;
pub use validation::*;
