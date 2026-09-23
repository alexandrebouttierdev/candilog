//! Contrats d'accès aux bibliothèques locales.

use super::{CoverLetter, NewCoverLetter, NewResume, ResumeSummary, ResumeVersion};
use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use uuid::Uuid;

pub trait ResumeRepository: Send + Sync {
    fn save(&self, input: &NewResume) -> AppResult<ResumeVersion>;
    /// Page de CV ; `scored_only` ne retient que ceux qui portent un score ATS
    /// (onglet « Analyses » de Documents).
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
        scored_only: bool,
    ) -> AppResult<Page<ResumeSummary>>;
    fn get(&self, id: Uuid) -> AppResult<ResumeVersion>;
    fn delete(&self, id: Uuid) -> AppResult<()>;
}

pub trait CoverLetterRepository: Send + Sync {
    fn save(&self, input: &NewCoverLetter) -> AppResult<CoverLetter>;
    fn list_page(&self, page: u64, page_size: u64, search: &str) -> AppResult<Page<CoverLetter>>;
    fn get(&self, id: Uuid) -> AppResult<CoverLetter>;
    fn delete(&self, id: Uuid) -> AppResult<()>;
}
