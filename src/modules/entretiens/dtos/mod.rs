//! Contrats d'entrée du domaine entretiens, un DTO par fichier.

mod create_entretien;
mod update_entretien;

pub use create_entretien::CreateEntretienDto;
pub use update_entretien::UpdateEntretienDto;
