//! Fusion du profil et de la lettre pour l'export PDF.

use crate::features::documents::domain::CoverLetterExport;
use crate::features::profile::domain::Profile;
use crate::infrastructure::pdf::CoverLetterPdf;

/// Identité du profil + objet et corps de la lettre.
#[must_use]
pub fn build_cover_letter(profile: &Profile, cover_letter: &CoverLetterExport) -> CoverLetterPdf {
    let identity = &profile.identity;
    let name = format!("{} {}", identity.first_name, identity.name)
        .trim()
        .to_owned();
    let subject = match (
        cover_letter.job_title.as_deref(),
        cover_letter.company.as_deref(),
    ) {
        (Some(job_title), Some(company))
            if !job_title.trim().is_empty() && !company.trim().is_empty() =>
        {
            format!("Objet : candidature au poste de {job_title} — {company}")
        }
        (Some(job_title), _) if !job_title.trim().is_empty() => {
            format!("Objet : candidature au poste de {job_title}")
        }
        _ => "Objet : candidature".into(),
    };
    CoverLetterPdf {
        name,
        city: identity.city.clone(),
        email: identity.email.clone(),
        subject,
        corps: cover_letter.content.clone(),
    }
}

#[cfg(test)]
#[path = "tests/lettre_document/mod.rs"]
mod tests;
