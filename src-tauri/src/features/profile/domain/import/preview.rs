use super::super::{Identity, Profile};
use super::matching::list_items;
use super::models::*;
use super::normalization::*;
use crate::core::errors::{AppError, AppResult};

const IDENTITY_FIELDS: &[(&str, &str)] = &[
    ("first_name", "Prénom"),
    ("name", "Nom"),
    ("email", "E-mail"),
    ("phone", "Téléphone"),
    ("city", "Ville"),
    ("title", "Titre professionnel"),
    ("resume", "Résumé"),
    ("linkedin", "LinkedIn"),
    ("github", "GitHub"),
    ("website", "Site web"),
];

/// Construit la preview à partir du profil actuel et des données extraites.
#[must_use]
pub fn build_preview(current: &Profile, extracted: &Profile) -> ImportProfilePreview {
    let identity = identity_items(&current.identity, &extracted.identity);
    let experiences: Vec<ImportExperienceItem> = list_items(
        "exp",
        &extracted.experiences,
        &current.experiences,
        experience_key,
    )
    .into_iter()
    .map(|item| ImportExperienceItem {
        id: item.id,
        proposed: item.proposed,
        existing: item.existing,
        existing_index: item.existing_index,
        has_conflict: item.has_conflict,
    })
    .collect::<Vec<_>>();
    let skills: Vec<ImportSkillItem> =
        list_items("skill", &extracted.skills, &current.skills, skill_key)
            .into_iter()
            .map(|item| ImportSkillItem {
                id: item.id,
                proposed: item.proposed,
                existing: item.existing,
                existing_index: item.existing_index,
                has_conflict: item.has_conflict,
            })
            .collect();
    let education: Vec<ImportEducationItem> = list_items(
        "edu",
        &extracted.education,
        &current.education,
        education_key,
    )
    .into_iter()
    .map(|item| ImportEducationItem {
        id: item.id,
        proposed: item.proposed,
        existing: item.existing,
        existing_index: item.existing_index,
        has_conflict: item.has_conflict,
    })
    .collect();
    let languages: Vec<ImportLanguageItem> = list_items(
        "lang",
        &extracted.languages,
        &current.languages,
        language_key,
    )
    .into_iter()
    .map(|item| ImportLanguageItem {
        id: item.id,
        proposed: item.proposed,
        existing: item.existing,
        existing_index: item.existing_index,
        has_conflict: item.has_conflict,
    })
    .collect();
    let projects: Vec<ImportProjectItem> =
        list_items("proj", &extracted.projects, &current.projects, project_key)
            .into_iter()
            .map(|item| ImportProjectItem {
                id: item.id,
                proposed: item.proposed,
                existing: item.existing,
                existing_index: item.existing_index,
                has_conflict: item.has_conflict,
            })
            .collect();
    let certifications: Vec<ImportCertificationItem> = list_items(
        "cert",
        &extracted.certifications,
        &current.certifications,
        certification_key,
    )
    .into_iter()
    .map(|item| ImportCertificationItem {
        id: item.id,
        proposed: item.proposed,
        existing: item.existing,
        existing_index: item.existing_index,
        has_conflict: item.has_conflict,
    })
    .collect();
    let counts = ImportDetectedCounts {
        identity: identity.len() as u32,
        experiences: experiences.len() as u32,
        skills: skills.len() as u32,
        education: education.len() as u32,
        languages: languages.len() as u32,
        projects: projects.len() as u32,
        certifications: certifications.len() as u32,
    };
    ImportProfilePreview {
        identity,
        experiences,
        skills,
        education,
        languages,
        projects,
        certifications,
        counts,
    }
}

fn identity_items(current: &Identity, extracted: &Identity) -> Vec<ImportScalarItem> {
    IDENTITY_FIELDS
        .iter()
        .filter_map(|(id, label)| {
            let proposed = identity_value(extracted, id)?;
            if proposed.trim().is_empty() {
                return None;
            }
            let existing = identity_value(current, id).filter(|value| !value.trim().is_empty());
            let has_conflict = existing
                .as_ref()
                .is_some_and(|value| !same_text(value, &proposed));
            Some(ImportScalarItem {
                id: (*id).to_owned(),
                label: (*label).to_owned(),
                proposed,
                existing,
                has_conflict,
            })
        })
        .collect()
}

pub(super) fn identity_value(identity: &Identity, id: &str) -> Option<String> {
    let value = match id {
        "first_name" => identity.first_name.clone(),
        "name" => identity.name.clone(),
        "email" => identity.email.clone(),
        "phone" => identity.phone.clone().unwrap_or_default(),
        "city" => identity.city.clone().unwrap_or_default(),
        "title" => identity.title.clone().unwrap_or_default(),
        "resume" => identity.resume.clone().unwrap_or_default(),
        "linkedin" => identity.linkedin.clone().unwrap_or_default(),
        "github" => identity.github.clone().unwrap_or_default(),
        "website" => identity.website.clone().unwrap_or_default(),
        _ => return None,
    };
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

pub(super) fn set_identity_field(identity: &mut Identity, id: &str, value: &str) -> AppResult<()> {
    let trimmed = value.trim();
    match id {
        "first_name" => identity.first_name = trimmed.to_owned(),
        "name" => identity.name = trimmed.to_owned(),
        "email" => identity.email = trimmed.to_owned(),
        "phone" => identity.phone = empty_to_none(trimmed),
        "city" => identity.city = empty_to_none(trimmed),
        "title" => identity.title = empty_to_none(trimmed),
        "resume" => identity.resume = empty_to_none(trimmed),
        "linkedin" => identity.linkedin = empty_to_none(trimmed),
        "github" => identity.github = empty_to_none(trimmed),
        "website" => identity.website = empty_to_none(trimmed),
        _ => {
            return Err(AppError::Validation(format!(
                "Champ d'identité inconnu : {id}"
            )));
        }
    }
    Ok(())
}
