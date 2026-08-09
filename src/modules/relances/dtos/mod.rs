//! Contrats d'entrée du domaine relances, un DTO par fichier.

mod create_relance;
mod update_relance;

pub use create_relance::CreateRelanceDto;
pub use update_relance::UpdateRelanceDto;
