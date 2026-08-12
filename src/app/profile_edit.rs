//! Mutations structurées des brouillons de profil et des recommandations ATS.

use super::state::ProfileCollection;
use crate::shared::profile::{Certification, Education, Experience, Language, Profile, Project};

fn key(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Fusionne un profil extrait avec le profil courant sans écraser les données existantes.
/// Les champs personnels déjà renseignés restent prioritaires ; les collections sont
/// dédoublonnées sur leur identité métier.
pub(super) fn merge_imported_profile(current: &Profile, imported: &Profile) -> Profile {
    let mut merged = current.clone();
    if merged.personal.first_name.trim().is_empty() {
        merged
            .personal
            .first_name
            .clone_from(&imported.personal.first_name);
    }
    if merged.personal.last_name.trim().is_empty() {
        merged
            .personal
            .last_name
            .clone_from(&imported.personal.last_name);
    }
    if merged.personal.email.trim().is_empty() {
        merged.personal.email.clone_from(&imported.personal.email);
    }
    macro_rules! fill_optional {
        ($field:ident) => {
            if merged.personal.$field.is_none() {
                merged.personal.$field.clone_from(&imported.personal.$field);
            }
        };
    }
    fill_optional!(phone);
    fill_optional!(city);
    fill_optional!(headline);
    fill_optional!(summary);
    fill_optional!(linkedin);
    fill_optional!(github);
    fill_optional!(website);

    for item in &imported.experiences {
        if !item.is_complete()
            || merged.experiences.iter().any(|existing| {
                key(&existing.title) == key(&item.title)
                    && key(&existing.company) == key(&item.company)
            })
        {
            continue;
        }
        merged.experiences.push(item.clone());
    }
    for item in &imported.skills {
        if item.is_complete()
            && !merged
                .skills
                .iter()
                .any(|existing| key(&existing.name) == key(&item.name))
        {
            merged.skills.push(item.clone());
        }
    }
    for item in &imported.education {
        if item.is_complete()
            && !merged.education.iter().any(|existing| {
                key(&existing.degree) == key(&item.degree)
                    && key(&existing.school) == key(&item.school)
            })
        {
            merged.education.push(item.clone());
        }
    }
    for item in &imported.languages {
        if item.is_complete()
            && !merged
                .languages
                .iter()
                .any(|existing| key(&existing.name) == key(&item.name))
        {
            merged.languages.push(item.clone());
        }
    }
    for item in &imported.projects {
        if item.is_complete()
            && !merged
                .projects
                .iter()
                .any(|existing| key(&existing.name) == key(&item.name))
        {
            merged.projects.push(item.clone());
        }
    }
    for item in &imported.certifications {
        if item.is_complete()
            && !merged
                .certifications
                .iter()
                .any(|existing| key(&existing.name) == key(&item.name))
        {
            merged.certifications.push(item.clone());
        }
    }
    merged
}

pub(crate) fn import_item_key(category: &str, index: usize) -> String {
    format!("{category}:{index}")
}

pub(super) fn all_import_item_keys(profile: &Profile) -> std::collections::HashSet<String> {
    let mut keys = std::collections::HashSet::new();
    macro_rules! personal_key_if_present {
        ($field:ident) => {
            if !profile.personal.$field.trim().is_empty() {
                keys.insert(import_item_key(concat!("personal.", stringify!($field)), 0));
            }
        };
    }
    macro_rules! optional_personal_key_if_present {
        ($field:ident) => {
            if profile
                .personal
                .$field
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            {
                keys.insert(import_item_key(concat!("personal.", stringify!($field)), 0));
            }
        };
    }
    personal_key_if_present!(first_name);
    personal_key_if_present!(last_name);
    personal_key_if_present!(email);
    optional_personal_key_if_present!(phone);
    optional_personal_key_if_present!(city);
    optional_personal_key_if_present!(headline);
    optional_personal_key_if_present!(summary);
    optional_personal_key_if_present!(linkedin);
    optional_personal_key_if_present!(github);
    optional_personal_key_if_present!(website);
    for (category, len) in [
        ("experiences", profile.experiences.len()),
        ("skills", profile.skills.len()),
        ("education", profile.education.len()),
        ("languages", profile.languages.len()),
        ("projects", profile.projects.len()),
        ("certifications", profile.certifications.len()),
    ] {
        keys.extend((0..len).map(|index| import_item_key(category, index)));
    }
    keys
}

pub(super) fn filter_imported_profile(
    profile: &Profile,
    excluded: &std::collections::HashSet<String>,
) -> Profile {
    let mut filtered = profile.clone();
    macro_rules! remove_personal_if_excluded {
        ($field:ident) => {
            if excluded.contains(&import_item_key(
                concat!("personal.", stringify!($field)),
                0,
            )) {
                filtered.personal.$field = Default::default();
            }
        };
    }
    remove_personal_if_excluded!(first_name);
    remove_personal_if_excluded!(last_name);
    remove_personal_if_excluded!(email);
    remove_personal_if_excluded!(phone);
    remove_personal_if_excluded!(city);
    remove_personal_if_excluded!(headline);
    remove_personal_if_excluded!(summary);
    remove_personal_if_excluded!(linkedin);
    remove_personal_if_excluded!(github);
    remove_personal_if_excluded!(website);
    filtered.experiences = profile
        .experiences
        .iter()
        .enumerate()
        .filter(|(index, _)| !excluded.contains(&import_item_key("experiences", *index)))
        .map(|(_, item)| item.clone())
        .collect();
    filtered.skills = profile
        .skills
        .iter()
        .enumerate()
        .filter(|(index, _)| !excluded.contains(&import_item_key("skills", *index)))
        .map(|(_, item)| item.clone())
        .collect();
    filtered.education = profile
        .education
        .iter()
        .enumerate()
        .filter(|(index, _)| !excluded.contains(&import_item_key("education", *index)))
        .map(|(_, item)| item.clone())
        .collect();
    filtered.languages = profile
        .languages
        .iter()
        .enumerate()
        .filter(|(index, _)| !excluded.contains(&import_item_key("languages", *index)))
        .map(|(_, item)| item.clone())
        .collect();
    filtered.projects = profile
        .projects
        .iter()
        .enumerate()
        .filter(|(index, _)| !excluded.contains(&import_item_key("projects", *index)))
        .map(|(_, item)| item.clone())
        .collect();
    filtered.certifications = profile
        .certifications
        .iter()
        .enumerate()
        .filter(|(index, _)| !excluded.contains(&import_item_key("certifications", *index)))
        .map(|(_, item)| item.clone())
        .collect();
    filtered
}

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

#[cfg(test)]
mod tests {
    use super::merge_imported_profile;
    use crate::shared::profile::{Experience, PersonalInfo, Profile, Skill};

    #[test]
    fn import_complete_les_vides_sans_ecraser_ni_dupliquer() {
        let current = Profile {
            personal: PersonalInfo {
                first_name: "Alexandre".into(),
                last_name: "Bouttier".into(),
                email: "alex@example.com".into(),
                ..PersonalInfo::default()
            },
            experiences: vec![Experience {
                title: "Technicien".into(),
                company: "ACME".into(),
                ..Experience::default()
            }],
            skills: vec![Skill {
                name: "Rust".into(),
            }],
            ..Profile::default()
        };
        let imported = Profile {
            personal: PersonalInfo {
                first_name: "Autre".into(),
                last_name: "Nom".into(),
                email: "autre@example.com".into(),
                phone: Some("0600000000".into()),
                ..PersonalInfo::default()
            },
            experiences: vec![
                Experience {
                    title: "Technicien".into(),
                    company: "ACME".into(),
                    ..Experience::default()
                },
                Experience {
                    title: "Administrateur".into(),
                    company: "Nouveau".into(),
                    ..Experience::default()
                },
            ],
            skills: vec![
                Skill {
                    name: "Rust".into(),
                },
                Skill {
                    name: "Linux".into(),
                },
            ],
            ..Profile::default()
        };

        let merged = merge_imported_profile(&current, &imported);
        assert_eq!(merged.personal.first_name, "Alexandre");
        assert_eq!(merged.personal.phone.as_deref(), Some("0600000000"));
        assert_eq!(merged.experiences.len(), 2);
        assert_eq!(merged.skills.len(), 2);
    }
}
