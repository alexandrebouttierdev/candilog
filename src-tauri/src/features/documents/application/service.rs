//! Validation des CV et lettres avant persistance.

use crate::core::errors::{AppError, AppResult};
use crate::features::documents::domain::{
    ResumeRepository, ResumeSummary, ResumeVersion, CoverLetter, CoverLetterRepository, NewResume, NewCoverLetter,
};
use uuid::Uuid;

pub struct DocumentsService<C: ResumeRepository, L: CoverLetterRepository> {
    resume: C,
    cover_letters: L,
}

impl<C: ResumeRepository, L: CoverLetterRepository> DocumentsService<C, L> {
    #[must_use]
    pub const fn new(resume: C, cover_letters: L) -> Self {
        Self { resume, cover_letters }
    }

    pub fn resume_save(&self, input: &NewResume) -> AppResult<ResumeVersion> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(AppError::Validation(
                "Le nom de la version est requis".into(),
            ));
        }
        if name.chars().count() > 120 {
            return Err(AppError::Validation(
                "Le nom de la version est trop long (120 max)".into(),
            ));
        }
        self.resume.save(&NewResume {
            name: name.into(),
            content: input.content.clone(),
        })
    }

    pub fn resume_list(&self) -> AppResult<Vec<ResumeSummary>> {
        self.resume.list()
    }
    pub fn resume_get(&self, id: Uuid) -> AppResult<ResumeVersion> {
        self.resume.get(id)
    }
    pub fn resume_delete(&self, id: Uuid) -> AppResult<()> {
        self.resume.delete(id)
    }

    pub fn cover_letter_save(&self, input: &NewCoverLetter) -> AppResult<CoverLetter> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(AppError::Validation(
                "Le nom de la lettre est requis".into(),
            ));
        }
        if name.chars().count() > 140 {
            return Err(AppError::Validation(
                "Le nom de la lettre est trop long".into(),
            ));
        }
        if input.content.trim().is_empty() {
            return Err(AppError::Validation(
                "Générez une lettre avant de l'enregistrer".into(),
            ));
        }
        let mut nettoyee = input.clone();
        nettoyee.name = name.into();
        self.cover_letters.save(&nettoyee)
    }

    pub fn cover_letters_list(&self) -> AppResult<Vec<CoverLetter>> {
        self.cover_letters.list()
    }
    pub fn cover_letter_get(&self, id: Uuid) -> AppResult<CoverLetter> {
        self.cover_letters.get(id)
    }
    pub fn cover_letter_delete(&self, id: Uuid) -> AppResult<()> {
        self.cover_letters.delete(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::{open_pool, run_local_migrations};
    use crate::features::documents::infrastructure::{SqliteResumeRepository, SqliteCoverLetterRepository};

    fn service() -> DocumentsService<SqliteResumeRepository, SqliteCoverLetterRepository> {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        DocumentsService::new(
            SqliteResumeRepository::new(pool.clone()),
            SqliteCoverLetterRepository::new(pool),
        )
    }

    #[test]
    fn refuse_un_cv_sans_nom() {
        let err = service()
            .resume_save(&NewResume {
                name: "   ".into(),
                content: serde_json::json!({}),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn refuse_une_lettre_sans_contenu() {
        let err = service()
            .cover_letter_save(&NewCoverLetter {
                name: "Lettre Nova".into(),
                company: Some("Nova".into()),
                job_title: Some("Designer".into()),
                tone: "formal".into(),
                length: "medium".into(),
                content: "  ".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
