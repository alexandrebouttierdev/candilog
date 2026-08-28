//! Génération de documents PDF autonomes (polices et icônes embarquées).

mod cv_pdf;

pub use cv_pdf::{CvEducation, CvExperience, CvLanguage, CvPdf, CvProject};
