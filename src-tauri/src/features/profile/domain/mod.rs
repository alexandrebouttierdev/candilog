//! Modèle métier du profil professionnel.

pub mod profile;
pub mod repository;

pub use profile::{
    Certification, Skill, Experience, Education, Identity, Language, Profile, ProfilePayload,
    Project,
};
pub use repository::ProfileRepository;
