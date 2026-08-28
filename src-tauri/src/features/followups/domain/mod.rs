//! Domaine des relances.

pub mod follow_up;
pub mod repository;

pub use follow_up::{NewFollowUp, FollowUp};
pub use repository::FollowUpRepository;
