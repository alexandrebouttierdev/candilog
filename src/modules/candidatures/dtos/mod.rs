//! Contrats d'entrée du domaine candidatures, un DTO par fichier.

mod create_candidature;
mod update_candidature;

pub use create_candidature::CreateCandidatureDto;
pub use update_candidature::UpdateCandidatureDto;
