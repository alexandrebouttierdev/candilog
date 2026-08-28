//! Contracts d'accès aux bibliothèques locales.

use super::{ResumeSummary, ResumeVersion, CoverLetter, NewResume, NewCoverLetter};
use crate::core::errors::AppResult;
use uuid::Uuid;

pub trait ResumeRepository: Send + Sync {
    fn save(&self, input: &NewResume) -> AppResult<ResumeVersion>;
    fn list(&self) -> AppResult<Vec<ResumeSummary>>;
    fn get(&self, id: Uuid) -> AppResult<ResumeVersion>;
    fn delete(&self, id: Uuid) -> AppResult<()>;
}

pub trait CoverLetterRepository: Send + Sync {
    fn save(&self, input: &NewCoverLetter) -> AppResult<CoverLetter>;
    fn list(&self) -> AppResult<Vec<CoverLetter>>;
    fn get(&self, id: Uuid) -> AppResult<CoverLetter>;
    fn delete(&self, id: Uuid) -> AppResult<()>;
}
