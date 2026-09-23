//! Domaine des contacts du réseau.

pub mod contact;
pub mod repository;

pub use contact::{Contact, ContactActivity, ContactUpdate, NewContact};
pub use repository::ContactRepository;
