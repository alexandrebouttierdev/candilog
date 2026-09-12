//! Contrats et modèles de l'assistance IA.

mod config;
mod cover_letter;
mod cv_analysis;
mod letter_natural;
mod managed_ollama;
mod models;
mod normalization;
mod profile_dates;
mod profile_grounding;
mod scoring;
mod system_resources;
mod user_benchmark;
mod validation;

pub use crate::core::utils::text::search_key;
pub use config::*;
pub use cover_letter::*;
pub use cv_analysis::*;
pub use letter_natural::*;
pub use managed_ollama::*;
pub use models::*;
pub use profile_dates::normalize_profile_dates;
pub use profile_grounding::{
    completer_contacts_vides, completer_descriptions_depuis_libelles,
    completer_formations_manquantes, completer_identite_noms, ground_imported_profile,
    ground_imported_profile_keep_free_text,
};
pub use scoring::{
    ground_ats_recommendations, ground_content_recommendations, ground_extracted_listing,
    ground_generated_resume, ground_imported_resume, profile_content_catalog, profile_score,
    score_resume_imported, score_resume_imported_with_source, ProfileContentCatalogEntry,
};
pub use system_resources::*;
pub use user_benchmark::{
    benchmark_ground_truth_path, benchmark_pdf_path, build_benchmark_result, load_ground_truth,
    score_extracted_profile, BenchmarkAnalysisOutcome, BenchmarkGroundTruth, BenchmarkScore,
};
pub use validation::*;
