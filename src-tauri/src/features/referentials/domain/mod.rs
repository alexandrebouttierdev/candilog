//! Domaine des référentiels métier.

pub mod catalog;
pub mod repository;

pub use catalog::{ActivitySector, ReferenceItem, Referentials};
pub use repository::ReferentialRepository;
