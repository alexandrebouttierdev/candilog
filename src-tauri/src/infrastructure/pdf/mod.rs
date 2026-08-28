//! Génération de documents PDF autonomes (polices et icônes embarquées).

mod resume_pdf;
mod cover_letter_pdf;

pub use resume_pdf::{ResumeEducation, ResumeExperience, ResumeLanguage, ResumePdf, ResumeProject};
pub use cover_letter_pdf::CoverLetterPdf;
