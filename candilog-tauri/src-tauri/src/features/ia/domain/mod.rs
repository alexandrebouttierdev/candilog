//! Contrats et modèles de l'assistance IA.

mod config;
mod models;
mod scoring;

pub use config::*;
pub use models::*;
pub use scoring::{score_cv_importe, score_profil};
