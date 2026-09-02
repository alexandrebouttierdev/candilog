//! Validation des champs et collections éditables du document.

use super::*;
use crate::core::errors::AppError;
use crate::features::ai::domain::{MAX_ITEMS, MAX_ITEM_CHARS};
use crate::features::documents::domain::{ResumeExperienceBlock, ResumeProjectBlock};

#[test]
fn refuse_un_nom_vide() {
    let mut document = minimal_document();
    document.identity.full_name = "   ".into();

    assert!(matches!(
        validate_document(&document),
        Err(AppError::Validation(message)) if message == "Le nom complet du CV est obligatoire."
    ));
}

#[test]
fn refuse_trop_de_competences() {
    let mut document = minimal_document();
    document.skill_groups[0].items = (0..=MAX_ITEMS)
        .map(|index| format!("Compétence {index}"))
        .collect();

    assert!(matches!(
        validate_document(&document),
        Err(AppError::Validation(message)) if message == "Un groupe de compétences contient trop d'éléments."
    ));
}

#[test]
fn refuse_trop_de_competences_reparties_dans_plusieurs_groupes() {
    let mut document = minimal_document();
    document.skill_groups = [51, 50]
        .into_iter()
        .enumerate()
        .map(|(group_index, count)| ResumeSkillGroup {
            id: format!("skills-{group_index}"),
            name: format!("Groupe {group_index}"),
            items: (0..count)
                .map(|item_index| format!("Compétence {group_index}-{item_index}"))
                .collect(),
        })
        .collect();

    assert!(matches!(
        validate_document(&document),
        Err(AppError::Validation(message)) if message == "Le CV contient trop de compétences."
    ));
}

#[test]
fn refuse_une_puce_trop_longue() {
    let mut document = minimal_document();
    document.experiences.push(ResumeExperienceBlock {
        id: "experience".into(),
        title: "Développeur".into(),
        company: "Exemple SARL".into(),
        location: None,
        period: "2024".into(),
        bullets: vec!["x".repeat(MAX_ITEM_CHARS + 1)],
    });

    assert!(matches!(
        validate_document(&document),
        Err(AppError::Validation(message)) if message == "Une puce du CV dépasse la taille maximale autorisée."
    ));
}

#[test]
fn refuse_les_url_non_http_du_document() {
    let mut document = minimal_document();
    document.identity.website = Some("javascript:alert(1)".into());
    assert!(matches!(
        validate_document(&document),
        Err(AppError::Validation(message)) if message == "Le site web du CV doit utiliser HTTP ou HTTPS"
    ));

    document.identity.website = Some("https://alex.example.test".into());
    document.projects.push(ResumeProjectBlock {
        id: "project".into(),
        name: "Projet".into(),
        meta: None,
        url: Some("data:text/html,attaque".into()),
        bullets: Vec::new(),
    });
    assert!(matches!(
        validate_document(&document),
        Err(AppError::Validation(message)) if message == "Le lien d'un projet du CV doit utiliser HTTP ou HTTPS"
    ));
}

#[test]
fn accepte_les_url_http_et_https_du_document() {
    let mut document = minimal_document();
    document.identity.website = Some("https://alex.example.test".into());
    document.identity.linkedin = Some("http://linkedin.example.test/alex".into());
    document.projects.push(ResumeProjectBlock {
        id: "project".into(),
        name: "Projet".into(),
        meta: None,
        url: Some("https://project.example.test".into()),
        bullets: Vec::new(),
    });

    validate_document(&document).unwrap();
}
