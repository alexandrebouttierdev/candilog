//! Génération de documents PDF autonomes (polices et icônes embarquées).

mod cover_letter_pdf;
mod page;
pub(crate) mod resume_pdf;

use crate::core::errors::{AppError, AppResult};

/// Compresse les flux d'un PDF produit par `printpdf`.
///
/// `printpdf` 0.12 sérialise polices et contenus **sans aucun filtre** : les cinq fontes
/// IBM Plex embarquées suffisaient à porter un CV d'une page à près d'un mégaoctet, refusé
/// par les portails de candidature qui plafonnent les pièces jointes. Le sous-ensemblage
/// des polices, qui ferait mieux encore, vit derrière la fonctionnalité `text_layout` de
/// `printpdf`, aujourd'hui inutilisable : elle réclame `azul-core 0.0.13`, absent de
/// crates.io. La compression est donc le gain disponible, et il divise le poids par deux.
///
/// # Errors
/// Retourne `Serialization` si le document produit n'est pas relisible ou réécrivable :
/// mieux vaut refuser l'export que livrer un fichier qu'aucun lecteur n'ouvrira.
pub(crate) fn compress(bytes: Vec<u8>) -> AppResult<Vec<u8>> {
    let mut document = lopdf::Document::load_mem(&bytes).map_err(|error| {
        tracing::error!(%error, "PDF produit illisible avant compression");
        AppError::Serialization("Le document PDF produit est illisible.".into())
    })?;
    document.compress();
    let mut compressed = Vec::with_capacity(bytes.len() / 2);
    document.save_to(&mut compressed).map_err(|error| {
        tracing::error!(%error, "compression du PDF impossible");
        AppError::Serialization("Le document PDF n'a pas pu être finalisé.".into())
    })?;
    Ok(compressed)
}

pub use cover_letter_pdf::CoverLetterPdf;
pub use page::{Density, LayoutBounds, Margins, PageSpec, A4, DENSITY_PROFILES};
pub use resume_pdf::{
    ResumeCertification, ResumeEducation, ResumeExperience, ResumeLanguage, ResumePdf,
    ResumePdfMeasurement, ResumeProject, ResumeSkillGroup,
};
