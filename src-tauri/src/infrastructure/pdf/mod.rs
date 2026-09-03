//! Génération de documents PDF autonomes (polices et icônes embarquées).

mod cover_letter_pdf;
mod page;
pub(crate) mod resume_pdf;

pub use cover_letter_pdf::CoverLetterPdf;
pub use page::{Density, LayoutBounds, Margins, PageSpec, A4, DENSITY_PROFILES};
pub use resume_pdf::{
    ResumeCertification, ResumeEducation, ResumeExperience, ResumeLanguage, ResumePdf,
    ResumeProject, ResumeSkillGroup,
};
