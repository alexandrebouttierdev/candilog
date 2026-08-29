//! Preview, conflits et application d'un import de profil depuis un CV.
//!
//! L'analyse produit uniquement une proposition. L'écriture n'a lieu qu'après
//! des décisions explicites.

use serde::{Deserialize, Serialize};

use crate::core::errors::{AppError, AppResult};

use super::{Certification, Education, Experience, Identity, Language, Profile, Project, Skill};

/// Décision utilisateur pour un élément en conflit ou à ajouter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub enum ImportResolution {
    KeepExisting,
    Replace,
    AddAsNew,
}

/// Champ d'identité proposé, éventuellement en conflit avec le profil actuel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportScalarItem {
    pub id: String,
    pub label: String,
    pub proposed: String,
    pub existing: Option<String>,
    pub has_conflict: bool,
}

macro_rules! import_list_types {
    ($item:ident, $decision:ident, $ty:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
        #[serde(rename_all = "snake_case")]
        #[ts(export, export_to = "profile.ts")]
        pub struct $item {
            pub id: String,
            pub proposed: $ty,
            pub existing: Option<$ty>,
            #[ts(type = "number | null")]
            pub existing_index: Option<u32>,
            pub has_conflict: bool,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
        #[serde(rename_all = "snake_case")]
        #[ts(export, export_to = "profile.ts")]
        pub struct $decision {
            pub id: String,
            pub selected: bool,
            pub value: $ty,
            #[ts(type = "number | null")]
            pub existing_index: Option<u32>,
            pub resolution: ImportResolution,
        }
    };
}

import_list_types!(ImportExperienceItem, ImportExperienceDecision, Experience);
import_list_types!(ImportSkillItem, ImportSkillDecision, Skill);
import_list_types!(ImportEducationItem, ImportEducationDecision, Education);
import_list_types!(ImportLanguageItem, ImportLanguageDecision, Language);
import_list_types!(ImportProjectItem, ImportProjectDecision, Project);
import_list_types!(
    ImportCertificationItem,
    ImportCertificationDecision,
    Certification
);

/// Compteurs des données détectées, pour le résumé de revue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportDetectedCounts {
    pub identity: u32,
    pub experiences: u32,
    pub skills: u32,
    pub education: u32,
    pub languages: u32,
    pub projects: u32,
    pub certifications: u32,
}

/// Proposition d'import : jamais persistée telle quelle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportProfilePreview {
    pub identity: Vec<ImportScalarItem>,
    pub experiences: Vec<ImportExperienceItem>,
    pub skills: Vec<ImportSkillItem>,
    pub education: Vec<ImportEducationItem>,
    pub languages: Vec<ImportLanguageItem>,
    pub projects: Vec<ImportProjectItem>,
    pub certifications: Vec<ImportCertificationItem>,
    pub counts: ImportDetectedCounts,
}

/// Décision sur un champ d'identité.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportScalarDecision {
    pub id: String,
    pub selected: bool,
    pub value: String,
    pub resolution: ImportResolution,
}

/// Requête d'application : uniquement des décisions validées par l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportProfileRequest {
    pub identity: Vec<ImportScalarDecision>,
    pub experiences: Vec<ImportExperienceDecision>,
    pub skills: Vec<ImportSkillDecision>,
    pub education: Vec<ImportEducationDecision>,
    pub languages: Vec<ImportLanguageDecision>,
    pub projects: Vec<ImportProjectDecision>,
    pub certifications: Vec<ImportCertificationDecision>,
}

/// Résultat d'un import appliqué en une écriture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportProfileResult {
    pub added: u32,
    pub replaced: u32,
    pub skipped: u32,
}

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

/// Applique les décisions sur une copie du profil. N'écrit pas.
pub fn apply_decisions(
    current: &Profile,
    request: &ImportProfileRequest,
) -> AppResult<(Profile, ImportProfileResult)> {
    let mut merged = current.clone();
    let mut added = 0;
    let mut replaced = 0;
    let mut skipped = 0;

    for decision in &request.identity {
        if !decision.selected || decision.resolution == ImportResolution::KeepExisting {
            skipped += 1;
            continue;
        }
        if decision.resolution == ImportResolution::AddAsNew {
            return Err(AppError::Validation(
                "Une information personnelle ne peut pas être importée comme nouvel élément".into(),
            ));
        }
        let had_value = identity_value(&merged.identity, &decision.id)
            .is_some_and(|value| !value.trim().is_empty());
        set_identity_field(&mut merged.identity, &decision.id, &decision.value)?;
        if had_value {
            replaced += 1;
        } else {
            added += 1;
        }
    }

    apply_list(
        &mut merged.experiences,
        request.experiences.iter().map(|item| ListChoice {
            selected: item.selected,
            value: item.value.clone(),
            existing_index: item.existing_index,
            resolution: item.resolution,
        }),
        &mut added,
        &mut replaced,
        &mut skipped,
    );
    apply_list(
        &mut merged.skills,
        request.skills.iter().map(|item| ListChoice {
            selected: item.selected,
            value: item.value.clone(),
            existing_index: item.existing_index,
            resolution: item.resolution,
        }),
        &mut added,
        &mut replaced,
        &mut skipped,
    );
    apply_list(
        &mut merged.education,
        request.education.iter().map(|item| ListChoice {
            selected: item.selected,
            value: item.value.clone(),
            existing_index: item.existing_index,
            resolution: item.resolution,
        }),
        &mut added,
        &mut replaced,
        &mut skipped,
    );
    apply_list(
        &mut merged.languages,
        request.languages.iter().map(|item| ListChoice {
            selected: item.selected,
            value: item.value.clone(),
            existing_index: item.existing_index,
            resolution: item.resolution,
        }),
        &mut added,
        &mut replaced,
        &mut skipped,
    );
    apply_list(
        &mut merged.projects,
        request.projects.iter().map(|item| ListChoice {
            selected: item.selected,
            value: item.value.clone(),
            existing_index: item.existing_index,
            resolution: item.resolution,
        }),
        &mut added,
        &mut replaced,
        &mut skipped,
    );
    apply_list(
        &mut merged.certifications,
        request.certifications.iter().map(|item| ListChoice {
            selected: item.selected,
            value: item.value.clone(),
            existing_index: item.existing_index,
            resolution: item.resolution,
        }),
        &mut added,
        &mut replaced,
        &mut skipped,
    );

    Ok((
        merged,
        ImportProfileResult {
            added,
            replaced,
            skipped,
        },
    ))
}

struct ListChoice<T> {
    selected: bool,
    value: T,
    existing_index: Option<u32>,
    resolution: ImportResolution,
}

struct ListMatch<T> {
    id: String,
    proposed: T,
    existing: Option<T>,
    existing_index: Option<u32>,
    has_conflict: bool,
}

fn apply_list<T: Clone>(
    target: &mut Vec<T>,
    decisions: impl IntoIterator<Item = ListChoice<T>>,
    added: &mut u32,
    replaced: &mut u32,
    skipped: &mut u32,
) {
    for decision in decisions {
        if !decision.selected || decision.resolution == ImportResolution::KeepExisting {
            *skipped += 1;
            continue;
        }
        if decision.resolution == ImportResolution::Replace {
            if let Some(index) = decision.existing_index {
                if let Some(slot) = target.get_mut(index as usize) {
                    *slot = decision.value;
                    *replaced += 1;
                    continue;
                }
            }
        }
        target.push(decision.value);
        *added += 1;
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

fn list_items<T: Clone>(
    prefix: &str,
    extracted: &[T],
    current: &[T],
    key: impl Fn(&T) -> String,
) -> Vec<ListMatch<T>> {
    let mut used = vec![false; current.len()];
    extracted
        .iter()
        .enumerate()
        .map(|(index, proposed)| {
            let proposed_key = key(proposed);
            let match_index = current.iter().enumerate().find_map(|(i, item)| {
                if used[i] || key(item) != proposed_key || proposed_key.is_empty() {
                    None
                } else {
                    Some(i)
                }
            });
            if let Some(i) = match_index {
                used[i] = true;
            }
            ListMatch {
                id: format!("{prefix}-{index}"),
                proposed: proposed.clone(),
                existing: match_index.map(|i| current[i].clone()),
                existing_index: match_index.map(|i| i as u32),
                has_conflict: match_index.is_some(),
            }
        })
        .collect()
}

fn identity_value(identity: &Identity, id: &str) -> Option<String> {
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

fn set_identity_field(identity: &mut Identity, id: &str, value: &str) -> AppResult<()> {
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

fn empty_to_none(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn same_text(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

fn experience_key(item: &Experience) -> String {
    normalize(&format!(
        "{}|{}|{}",
        item.title, item.company, item.start_date
    ))
}

fn skill_key(item: &Skill) -> String {
    normalize(&item.name)
}

fn education_key(item: &Education) -> String {
    normalize(&format!("{}|{}", item.degree, item.school))
}

fn language_key(item: &Language) -> String {
    normalize(&item.name)
}

fn project_key(item: &Project) -> String {
    normalize(&item.name)
}

fn certification_key(item: &Certification) -> String {
    normalize(&item.name)
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vide() -> Profile {
        Profile::default()
    }

    fn experience(title: &str, company: &str, start: &str) -> Experience {
        Experience {
            title: title.into(),
            company: company.into(),
            start_date: start.into(),
            current: true,
            ..Experience::default()
        }
    }

    #[test]
    fn preview_ignore_les_champs_identite_vides() {
        let extracted = Profile {
            identity: Identity {
                first_name: "Camille".into(),
                ..Identity::default()
            },
            ..vide()
        };

        let preview = build_preview(&vide(), &extracted);

        assert_eq!(preview.counts.identity, 1);
        assert_eq!(preview.identity[0].id, "first_name");
        assert!(!preview.identity[0].has_conflict);
    }

    #[test]
    fn preview_signale_un_conflit_identite() {
        let current = Profile {
            identity: Identity {
                title: Some("Développeuse".into()),
                ..Identity::default()
            },
            ..vide()
        };
        let extracted = Profile {
            identity: Identity {
                title: Some("Développeuse frontend".into()),
                ..Identity::default()
            },
            ..vide()
        };

        let preview = build_preview(&current, &extracted);

        assert!(preview.identity[0].has_conflict);
        assert_eq!(
            preview.identity[0].existing.as_deref(),
            Some("Développeuse")
        );
    }

    #[test]
    fn preview_detecte_une_experience_similaire() {
        let current = Profile {
            experiences: vec![experience("Dev", "Lumen", "2022-03")],
            ..vide()
        };
        let extracted = Profile {
            experiences: vec![Experience {
                title: "Dev".into(),
                company: "Lumen".into(),
                start_date: "2022-03".into(),
                description: Some("Lead".into()),
                current: true,
                ..Experience::default()
            }],
            ..vide()
        };

        let preview = build_preview(&current, &extracted);

        assert!(preview.experiences[0].has_conflict);
        assert_eq!(preview.experiences[0].existing_index, Some(0));
    }

    #[test]
    fn conserver_l_existant_ne_modifie_pas_le_profil() {
        let current = Profile {
            identity: Identity {
                title: Some("Développeuse".into()),
                ..Identity::default()
            },
            skills: vec![Skill {
                name: "Rust".into(),
            }],
            ..vide()
        };
        let request = ImportProfileRequest {
            identity: vec![ImportScalarDecision {
                id: "title".into(),
                selected: true,
                value: "Lead".into(),
                resolution: ImportResolution::KeepExisting,
            }],
            experiences: vec![],
            skills: vec![ImportSkillDecision {
                id: "skill-0".into(),
                selected: true,
                value: Skill {
                    name: "React".into(),
                },
                existing_index: Some(0),
                resolution: ImportResolution::KeepExisting,
            }],
            education: vec![],
            languages: vec![],
            projects: vec![],
            certifications: vec![],
        };

        let (merged, result) = apply_decisions(&current, &request).unwrap();

        assert_eq!(merged, current);
        assert_eq!(result.skipped, 2);
        assert_eq!(result.added, 0);
        assert_eq!(result.replaced, 0);
    }

    #[test]
    fn remplacer_ecrase_uniquement_apres_decision() {
        let current = Profile {
            identity: Identity {
                title: Some("Développeuse".into()),
                ..Identity::default()
            },
            experiences: vec![experience("Dev", "Lumen", "2022-03")],
            ..vide()
        };
        let request = ImportProfileRequest {
            identity: vec![ImportScalarDecision {
                id: "title".into(),
                selected: true,
                value: "Lead".into(),
                resolution: ImportResolution::Replace,
            }],
            experiences: vec![ImportExperienceDecision {
                id: "exp-0".into(),
                selected: true,
                value: experience("Dev senior", "Lumen", "2022-03"),
                existing_index: Some(0),
                resolution: ImportResolution::Replace,
            }],
            skills: vec![],
            education: vec![],
            languages: vec![],
            projects: vec![],
            certifications: vec![],
        };

        let (merged, result) = apply_decisions(&current, &request).unwrap();

        assert_eq!(merged.identity.title.as_deref(), Some("Lead"));
        assert_eq!(merged.experiences[0].title, "Dev senior");
        assert_eq!(result.replaced, 2);
    }

    #[test]
    fn ajouter_comme_nouvel_element_conserve_l_existant() {
        let current = Profile {
            experiences: vec![experience("Dev", "Lumen", "2022-03")],
            ..vide()
        };
        let request = ImportProfileRequest {
            identity: vec![],
            experiences: vec![ImportExperienceDecision {
                id: "exp-0".into(),
                selected: true,
                value: experience("Dev", "Lumen", "2022-03"),
                existing_index: Some(0),
                resolution: ImportResolution::AddAsNew,
            }],
            skills: vec![],
            education: vec![],
            languages: vec![],
            projects: vec![],
            certifications: vec![],
        };

        let (merged, result) = apply_decisions(&current, &request).unwrap();

        assert_eq!(merged.experiences.len(), 2);
        assert_eq!(result.added, 1);
    }

    #[test]
    fn une_entree_non_selectionnee_est_ignoree() {
        let current = vide();
        let request = ImportProfileRequest {
            identity: vec![],
            experiences: vec![],
            skills: vec![ImportSkillDecision {
                id: "skill-0".into(),
                selected: false,
                value: Skill {
                    name: "Docker".into(),
                },
                existing_index: None,
                resolution: ImportResolution::AddAsNew,
            }],
            education: vec![],
            languages: vec![],
            projects: vec![],
            certifications: vec![],
        };

        let (merged, result) = apply_decisions(&current, &request).unwrap();

        assert!(merged.skills.is_empty());
        assert_eq!(result.skipped, 1);
    }

    #[test]
    fn add_as_new_est_refuse_pour_l_identite() {
        let request = ImportProfileRequest {
            identity: vec![ImportScalarDecision {
                id: "email".into(),
                selected: true,
                value: "a@example.fr".into(),
                resolution: ImportResolution::AddAsNew,
            }],
            experiences: vec![],
            skills: vec![],
            education: vec![],
            languages: vec![],
            projects: vec![],
            certifications: vec![],
        };

        let error = apply_decisions(&vide(), &request).unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
    }

    #[test]
    fn un_champ_identite_inconnu_est_refuse() {
        let request = ImportProfileRequest {
            identity: vec![ImportScalarDecision {
                id: "nickname".into(),
                selected: true,
                value: "Cam".into(),
                resolution: ImportResolution::Replace,
            }],
            experiences: vec![],
            skills: vec![],
            education: vec![],
            languages: vec![],
            projects: vec![],
            certifications: vec![],
        };

        let error = apply_decisions(&vide(), &request).unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
    }
}
