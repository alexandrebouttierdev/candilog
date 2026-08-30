//! Façade du domaine d'import : prévisualisation, conflits et application depuis un CV.
//!
//! L'analyse produit uniquement une proposition. L'écriture n'a lieu qu'après
//! des décisions explicites.

mod models;
use crate::core::errors::{AppError, AppResult};
pub use models::*;

use super::Profile;
mod matching;
mod normalization;
use matching::*;
mod preview;
pub use preview::build_preview;
use preview::{identity_value, set_identity_field};

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

#[cfg(test)]
mod tests {
    use super::super::{Experience, Identity, Skill};
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
