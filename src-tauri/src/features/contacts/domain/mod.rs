//! Domaine des contacts du réseau.

pub mod contact;
pub mod repository;

pub use contact::{Contact, MajContact, NouveauContact};
pub use repository::ContactRepository;
