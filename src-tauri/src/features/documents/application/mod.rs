//! Cas d'usage des bibliothèques Documents.

mod cv_document;
mod lettre_document;
mod service;
pub use cv_document::construire;
pub use lettre_document::construire_lettre;
pub use service::DocumentsService;
