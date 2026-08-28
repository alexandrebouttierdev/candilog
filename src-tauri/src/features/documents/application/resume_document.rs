//! Fusion du profil et du CV généré pour l'export PDF.

use crate::features::ai::domain::GeneratedResume;
use crate::features::profile::domain::Profile;
use crate::infrastructure::pdf::{ResumeEducation, ResumeExperience, ResumeLanguage, ResumePdf, ResumeProject};

/// Construit le modèle de CV, en fusionnant le profil (identité, coordonnées,
/// projets, langues, périodes) et le CV généré (contenu reformulé).
#[must_use]
pub fn build(profile: &Profile, generation: &GeneratedResume) -> ResumePdf {
    let identity = &profile.identity;
    let mut resume = ResumePdf {
        name: format!("{} {}", identity.first_name, identity.name)
            .trim()
            .to_owned(),
        subtitle: identity.title.clone().unwrap_or_default(),
        phone: identity.phone.clone(),
        email: identity.email.clone(),
        city: identity.city.clone(),
        linkedin: identity.linkedin.clone(),
        website: identity.website.clone(),
        profile: generation.resume.clone(),
        skills: generation.skills.clone(),
        ..ResumePdf::default()
    };

    resume.experiences = generation
        .experiences
        .iter()
        .map(|experience| {
            let meta = profile
                .experiences
                .iter()
                .find(|e| e.title.trim() == experience.title.trim())
                .map(|e| {
                    let mut parts = Vec::new();
                    if let Some(location) = e.location.as_deref() {
                        if !location.trim().is_empty() {
                            parts.push(location.to_owned());
                        }
                    }
                    let period = formater_period(Some(&e.start_date), e.end_date.as_deref());
                    if !period.is_empty() {
                        parts.push(period);
                    }
                    parts.join(" · ")
                })
                .unwrap_or_default();
            ResumeExperience {
                title: experience.title.clone(),
                company: experience.company.clone(),
                meta,
                bullets: decouper_puces(&experience.description),
            }
        })
        .collect();

    resume.projects = profile
        .projects
        .iter()
        .map(|project| ResumeProject {
            name: project.name.clone(),
            meta: project.technologies.clone().unwrap_or_default(),
            bullets: project
                .description
                .as_deref()
                .map(decouper_puces)
                .unwrap_or_default(),
        })
        .collect();

    resume.education = generation
        .education
        .iter()
        .map(|education| {
            let date = profile
                .education
                .iter()
                .find(|e| e.degree.trim() == education.degree.trim())
                .map(|e| formater_period(e.start_date.as_deref(), e.end_date.as_deref()))
                .unwrap_or_default();
            ResumeEducation {
                degree: education.degree.clone(),
                school: education.school.clone(),
                date,
            }
        })
        .collect();

    resume.languages = profile
        .languages
        .iter()
        .map(|language| ResumeLanguage {
            name: language.name.clone(),
            level: language.level.clone(),
        })
        .collect();

    resume
}

/// Découpe une description en puces : une ligne non vide = une puce, les
/// marqueurs courants (`·`, `-`, `•`) étant retirés en tête de ligne.
fn decouper_puces(description: &str) -> Vec<String> {
    description
        .lines()
        .map(|row| {
            row
                .trim()
                .trim_start_matches(['·', '-', '•', '*', ' '])
                .trim()
                .to_owned()
        })
        .filter(|row| !row.is_empty())
        .collect()
}

/// Formate une période « début – fin » en français.
fn formater_period(start: Option<&str>, end: Option<&str>) -> String {
    match (start, end) {
        (Some(start), Some(end)) => {
            format!(
                "{} – {}",
                formater_date_month(start),
                formater_date_month(end)
            )
        }
        (Some(start), None) => formater_date_month(start),
        (None, Some(end)) => formater_date_month(end),
        (None, None) => String::new(),
    }
}

/// Formate une date `AAAA-MM` en « Month. AAAA », ou `AAAA` telle quelle.
fn formater_date_month(value: &str) -> String {
    let Some((year, month)) = value.split_once('-') else {
        return value.to_owned();
    };
    let (Ok(year), Ok(month)) = (year.parse::<u32>(), month.parse::<u32>()) else {
        return value.to_owned();
    };
    format!("{} {year}", month_abrege(month))
}

/// Abréviation française d'un mois, de 1 à 12.
const fn month_abrege(number: u32) -> &'static str {
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
mod tests;
