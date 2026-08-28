//! Domaine des candidatures.

pub mod candidature;
pub mod repository;
pub mod statut;

pub use candidature::{Candidature, NouvelleCandidature};
pub use repository::{
    CandidatureRepository, FiltreCandidatures, RepartitionPipeline, TriCandidature,
};
pub use statut::{StatutCandidature, TypeContrat};
