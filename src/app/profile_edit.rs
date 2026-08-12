//! Mutations structurées des brouillons de profil et des recommandations ATS.

use super::state::ProfileCollection;
use crate::shared::profile::{Certification, Education, Experience, Language, Profile, Project};

fn optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

pub(super) fn add_profile_item(profile: &mut Profile, kind: ProfileCollection) {
    match kind {
        ProfileCollection::Experience => profile.experiences.push(Experience::default()),
        ProfileCollection::Formation => profile.education.push(Education::default()),
        ProfileCollection::Langue => profile.languages.push(Language::default()),
        ProfileCollection::Projet => profile.projects.push(Project::default()),
        ProfileCollection::Certification => profile.certifications.push(Certification::default()),
    }
}

pub(super) fn remove_profile_item(profile: &mut Profile, kind: ProfileCollection, index: usize) {
    match kind {
        ProfileCollection::Experience if index < profile.experiences.len() => {
            profile.experiences.remove(index);
        }
        ProfileCollection::Formation if index < profile.education.len() => {
            profile.education.remove(index);
        }
        ProfileCollection::Langue if index < profile.languages.len() => {
            profile.languages.remove(index);
        }
        ProfileCollection::Projet if index < profile.projects.len() => {
            profile.projects.remove(index);
        }
        ProfileCollection::Certification if index < profile.certifications.len() => {
            profile.certifications.remove(index);
        }
        _ => {}
    }
}

pub(super) fn update_profile_item(
    profile: &mut Profile,
    kind: ProfileCollection,
    index: usize,
    field: usize,
    value: String,
) {
    match kind {
        ProfileCollection::Experience => {
            if let Some(item) = profile.experiences.get_mut(index) {
                match field {
                    0 => item.title = value,
                    1 => item.company = value,
                    2 => item.location = optional(&value),
                    3 => item.start_date = value,
                    4 => {
                        item.end_date = optional(&value);
                        item.current = item.end_date.is_none();
                    }
                    5 => item.description = optional(&value),
                    _ => {}
                }
            }
        }
        ProfileCollection::Formation => {
            if let Some(item) = profile.education.get_mut(index) {
                match field {
                    0 => item.degree = value,
                    1 => item.school = value,
                    2 => item.location = optional(&value),
                    3 => item.start_date = optional(&value),
                    4 => item.end_date = optional(&value),
                    5 => item.description = optional(&value),
                    _ => {}
                }
            }
        }
        ProfileCollection::Langue => {
            if let Some(item) = profile.languages.get_mut(index) {
                match field {
                    0 => item.name = value,
                    1 => item.level = value,
                    _ => {}
                }
            }
        }
        ProfileCollection::Projet => {
            if let Some(item) = profile.projects.get_mut(index) {
                match field {
                    0 => item.name = value,
                    1 => item.url = optional(&value),
                    2 => item.technologies = optional(&value),
                    3 => item.description = optional(&value),
                    _ => {}
                }
            }
        }
        ProfileCollection::Certification => {
            if let Some(item) = profile.certifications.get_mut(index) {
                match field {
                    0 => item.name = value,
                    1 => item.issuer = optional(&value),
                    2 => item.date = optional(&value),
                    3 => item.url = optional(&value),
                    _ => {}
                }
            }
        }
    }
}

pub(super) fn apply_recommendation(
    cv: &mut crate::modules::ia::cv_model::GeneratedCv,
    recommendation: &crate::modules::ia::cv_model::RecommandationAts,
) {
    match recommendation.section.as_str() {
        "resume" | "summary" => cv.summary.clone_from(&recommendation.texte_propose),
        "competences" | "skills" => {
            cv.skills = recommendation
                .texte_propose
                .split([',', ';', '\n'])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect();
        }
        section if section.starts_with("experience_") => {
            let index = section
                .trim_start_matches("experience_")
                .parse::<usize>()
                .ok();
            if let Some(experience) = index.and_then(|index| cv.experiences.get_mut(index)) {
                experience
                    .description
                    .clone_from(&recommendation.texte_propose);
            }
        }
        _ => {}
    }
}
