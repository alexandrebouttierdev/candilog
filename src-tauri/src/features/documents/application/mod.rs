//! Cas d'usage des bibliothèques Documents.

mod cover_letter_document;
mod resume_document;
mod resume_workspace;
mod service;
pub use cover_letter_document::build_cover_letter;
pub use resume_document::{build, measure};
pub use resume_workspace::{
    apply_proposal, build_proposals, prepare_workspace, recalculate, reject_proposal,
    to_generated_resume, validate_document,
};
pub use service::DocumentsService;
