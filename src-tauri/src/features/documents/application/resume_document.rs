//! Construction du modèle PDF à partir d'un document autonome.

use crate::core::errors::AppResult;
use crate::features::documents::domain::{
    ResumeDocument, ResumeExperienceBlock, ResumeLayoutMeasurement, ResumeLayoutStatus,
    ResumeProjectBlock,
};
use crate::infrastructure::pdf::{
    ResumeCertification, ResumeEducation, ResumeExperience, ResumeLanguage, ResumePdf,
    ResumeProject, ResumeSkillGroup,
};

/// Construit le modèle de CV directement depuis le document enregistré.
///
/// `photo` porte les octets PNG de la photo du profil, ou `None` : le CV se compose
/// exactement de la même façon dans les deux cas, seul l'en-tête diffère.
#[must_use]
pub fn build(document: &ResumeDocument, photo: Option<Vec<u8>>) -> ResumePdf {
    let identity = &document.identity;
    ResumePdf {
        name: identity.full_name.clone(),
        subtitle: identity.title.clone(),
        headline: identity.headline.clone(),
        phone: identity.phone.clone(),
        email: identity.email.clone(),
        city: identity.city.clone(),
        linkedin: identity.linkedin.clone(),
        website: identity.website.clone(),
        github: identity.github.clone(),
        extra: identity.extra.clone(),
        profile: document.profile.clone(),
        skill_groups: document
            .skill_groups
            .iter()
            .map(|group| ResumeSkillGroup {
                name: group.name.clone(),
                items: group.items.clone(),
            })
            .collect(),
        experiences: document
            .experiences
            .iter()
            .map(experience_from_block)
            .collect(),
        projects: document.projects.iter().map(project_from_block).collect(),
        education: document
            .education
            .iter()
            .map(|education| ResumeEducation {
                degree: education.degree.clone(),
                school: education.school.clone(),
                location: education.location.clone(),
                period: education.period.clone(),
                description: education.description.clone(),
            })
            .collect(),
        certifications: document
            .certifications
            .iter()
            .map(|certification| ResumeCertification {
                name: certification.name.clone(),
                issuer: certification.issuer.clone(),
                date: certification.date.clone(),
            })
            .collect(),
        languages: document
            .languages
            .iter()
            .map(|language| ResumeLanguage {
                name: language.name.clone(),
                level: language.level.clone(),
            })
            .collect(),
        photo,
    }
}

/// Mesure le document par la chaîne de composition PDF, sans créer d'artefact.
///
/// # Errors
/// Retourne une erreur si les polices embarquées ne peuvent pas être décodées.
pub fn measure(
    document: &ResumeDocument,
    photo: Option<Vec<u8>>,
) -> AppResult<ResumeLayoutMeasurement> {
    let measured = build(document, photo).measure()?;
    let status = if measured.overflow {
        ResumeLayoutStatus::Overflow
    } else if measured.density_index >= 4 && measured.used_ratio >= 0.97 {
        ResumeLayoutStatus::Full
    } else if measured.density_index >= 3 || measured.used_ratio >= 0.90 {
        ResumeLayoutStatus::AlmostFull
    } else if measured.used_ratio >= 0.72 {
        ResumeLayoutStatus::Available
    } else {
        ResumeLayoutStatus::Spacious
    };
    let page_count = if measured.overflow {
        measured.used_ratio.ceil().clamp(2.0, f32::from(u8::MAX)) as u8
    } else {
        1
    };
    Ok(ResumeLayoutMeasurement {
        status,
        used_per_mille: (measured.used_ratio * 1_000.0).round().clamp(0.0, 2_000.0) as u16,
        remaining_points: measured.remaining_points.round() as i32,
        page_count,
        overflow: measured.overflow,
    })
}

fn experience_from_block(experience: &ResumeExperienceBlock) -> ResumeExperience {
    ResumeExperience {
        title: experience.title.clone(),
        company: experience.company.clone(),
        location: experience.location.clone(),
        period: experience.period.clone(),
        bullets: experience.bullets.clone(),
    }
}

fn project_from_block(project: &ResumeProjectBlock) -> ResumeProject {
    ResumeProject {
        name: project.name.clone(),
        meta: project.meta.clone().unwrap_or_default(),
        url: project.url.clone(),
        bullets: project.bullets.clone(),
    }
}

/// Découpe une description en puces : une ligne non vide = une puce, les
/// marqueurs courants (`·`, `-`, `•`) étant retirés en tête de ligne.
pub(super) fn split_bullets(description: &str) -> Vec<String> {
    description
        .lines()
        .map(|row| {
            row.trim()
                .trim_start_matches(['·', '-', '•', '*', ' '])
                .trim()
                .to_owned()
        })
        .filter(|row| !row.is_empty())
        .collect()
}

/// Formate une date `AAAA-MM` en « Month. AAAA », ou `AAAA` telle quelle.
pub(super) fn format_month_date(value: &str) -> String {
    let Some((year, month)) = value.split_once('-') else {
        return value.to_owned();
    };
    let (Ok(year), Ok(month)) = (year.parse::<u32>(), month.parse::<u32>()) else {
        return value.to_owned();
    };
    format!("{} {year}", abbreviated_month(month))
}

/// Abréviation française d'un mois, de 1 à 12.
const fn abbreviated_month(number: u32) -> &'static str {
    match number {
        1 => "Janv.",
        2 => "Févr.",
        3 => "Mars",
        4 => "Avr.",
        5 => "Mai",
        6 => "Juin",
        7 => "Juil.",
        8 => "Août",
        9 => "Sept.",
        10 => "Oct.",
        11 => "Nov.",
        12 => "Déc.",
        _ => "?",
    }
}

#[cfg(test)]
#[path = "tests/cv_document/mod.rs"]
mod cv_document;
