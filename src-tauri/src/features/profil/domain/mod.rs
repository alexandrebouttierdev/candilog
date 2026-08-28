//! Modèle métier du profil professionnel.

pub mod profil;
pub mod repository;

pub use profil::{
    Certification, Competence, Experience, Formation, Identite, Langue, Profil, ProfilCharge,
    Projet,
};
pub use repository::ProfilRepository;
