//! Contrats d'accès aux bibliothèques locales.

use super::{
    CoverLetter, DocumentVersion, NewCoverLetter, NewResume, ResumeSummary, ResumeVersion,
};
use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use uuid::Uuid;

pub trait ResumeRepository: Send + Sync {
    fn save(&self, input: &NewResume) -> AppResult<ResumeVersion>;
    /// Page des CV, version courante de chacun ; `scored_only` ne retient que ceux qui
    /// portent un score ATS (onglet « Analyses » de Documents).
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
        scored_only: bool,
    ) -> AppResult<Page<ResumeSummary>>;
    fn get(&self, id: Uuid) -> AppResult<ResumeVersion>;
    /// Versions du document auquel appartient `id`, de la plus récente à la plus ancienne.
    fn versions(&self, id: Uuid) -> AppResult<Vec<DocumentVersion>>;
    /// Fait de la version `id` la version courante de son document ; rien n'est effacé.
    fn restore(&self, id: Uuid) -> AppResult<()>;
    /// Supprime le document auquel appartient `id`, toutes versions comprises.
    fn delete(&self, id: Uuid) -> AppResult<()>;
}

pub trait CoverLetterRepository: Send + Sync {
    fn save(&self, input: &NewCoverLetter) -> AppResult<CoverLetter>;
    fn list_page(&self, page: u64, page_size: u64, search: &str) -> AppResult<Page<CoverLetter>>;
    fn get(&self, id: Uuid) -> AppResult<CoverLetter>;
    /// Versions du document auquel appartient `id`, de la plus récente à la plus ancienne.
    fn versions(&self, id: Uuid) -> AppResult<Vec<DocumentVersion>>;
    /// Fait de la version `id` la version courante de son document ; rien n'est effacé.
    fn restore(&self, id: Uuid) -> AppResult<()>;
    /// Supprime le document auquel appartient `id`, toutes versions comprises.
    fn delete(&self, id: Uuid) -> AppResult<()>;
}
