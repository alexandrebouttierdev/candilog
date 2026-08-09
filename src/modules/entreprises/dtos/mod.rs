//! Contrats d'entrée du domaine entreprises, un DTO par fichier.

mod create_entreprise;
mod update_entreprise;

pub use create_entreprise::CreateEntrepriseDto;
pub use update_entreprise::UpdateEntrepriseDto;
