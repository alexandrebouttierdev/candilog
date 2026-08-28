//! Cas d'usage des bibliothèques Documents.

mod resume_document;
mod cover_letter_document;
mod service;
pub use resume_document::build;
pub use cover_letter_document::build_cover_letter;
pub use service::DocumentsService;
