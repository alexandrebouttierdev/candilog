//! Génération de documents PDF autonomes (polices et icônes embarquées).

mod cover_letter_pdf;
mod resume_pdf;

pub use cover_letter_pdf::CoverLetterPdf;
pub use resume_pdf::{ResumeEducation, ResumeExperience, ResumeLanguage, ResumePdf, ResumeProject};
