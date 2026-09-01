//! Fusion du profil et de la lettre pour l'export PDF.

use crate::features::documents::domain::CoverLetterExport;
use crate::features::profile::domain::Profile;
use crate::infrastructure::pdf::CoverLetterPdf;

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// Identité du profil + destinataire et corps de la lettre.
#[must_use]
pub fn build_cover_letter(profile: &Profile, cover_letter: &CoverLetterExport) -> CoverLetterPdf {
    let identity = &profile.identity;
    CoverLetterPdf {
        first_name: identity.first_name.clone(),
        last_name: identity.name.clone(),
        title: optional_text(identity.title.as_deref()),
        address: optional_text(identity.address.as_deref()),
        city: optional_text(identity.city.as_deref()),
        phone: optional_text(identity.phone.as_deref()),
        email: identity.email.clone(),
        company: optional_text(cover_letter.company.as_deref()),
        recipient: optional_text(cover_letter.recipient.as_deref()),
        recipient_address: optional_text(cover_letter.recipient_address.as_deref()),
        job_title: optional_text(cover_letter.job_title.as_deref()),
        job_reference: optional_text(cover_letter.job_reference.as_deref()),
        corps: cover_letter.content.clone(),
    }
}

#[cfg(test)]
#[path = "tests/lettre_document/mod.rs"]
mod tests;
