//! Domaine des entreprises.

pub mod entreprise;
pub mod repository;

pub use entreprise::{Entreprise, MajEntreprise, NouvelleEntreprise};
pub use repository::EntrepriseRepository;
