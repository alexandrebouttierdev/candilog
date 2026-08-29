//! Cas d'usage des bibliothèques Documents.

mod cover_letter_document;
mod resume_document;
mod service;
pub use cover_letter_document::build_cover_letter;
pub use resume_document::build;
pub use service::DocumentsService;
