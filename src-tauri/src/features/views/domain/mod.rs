//! Domaine des vues enregistrées.

pub mod repository;
pub mod saved_view;

pub use repository::SavedViewRepository;
pub use saved_view::{NewSavedView, SavedView, MAX_VIEW_NAME};
