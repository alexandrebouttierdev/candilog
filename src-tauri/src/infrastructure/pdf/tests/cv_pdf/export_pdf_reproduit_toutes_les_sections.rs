//! Un document complet doit reproduire chaque section du template.

use super::*;
use crate::features::documents::application::build;
use crate::features::documents::domain::{
    ResumeCertificationBlock, ResumeDocument, ResumeEducationBlock, ResumeExperienceBlock,
    ResumeIdentity, ResumeLanguageBlock, ResumeProjectBlock, ResumeSkillGroup,
};

fn document_complet() -> ResumeDocument {
    ResumeDocument {
        identity: ResumeIdentity {
            full_name: "Alex Exemple".into(),
            title: "Administrateur systèmes".into(),
            email: "alex@exemple.fr".into(),
            ..ResumeIdentity::default()
        },
        profile: "Profil autonome de test.".into(),
        experiences: vec![ResumeExperienceBlock {
            id: "exp-1".into(),
            title: "Ingénieur".into(),
            company: "Candilog".into(),
            location: Some("Rennes".into()),
            period: "2024 — Aujourd'hui".into(),
            bullets: vec!["Mission principale.".into()],
        }],
        projects: vec![ResumeProjectBlock {
            id: "proj-1".into(),
            name: "Atlas".into(),
            meta: Some("Open source".into()),
            url: Some("https://atlas.dev".into()),
            bullets: vec!["Composants accessibles.".into()],
        }],
        skill_groups: vec![ResumeSkillGroup {
            id: "skills".into(),
            name: "Développement".into(),
            items: vec!["Rust".into()],
        }],
        education: vec![ResumeEducationBlock {
            id: "edu-1".into(),
            degree: "Master".into(),
            school: "Université".into(),
            location: Some("Rennes".into()),
            period: "2020".into(),
            description: Some("Spécialité logicielle.".into()),
        }],
        certifications: vec![ResumeCertificationBlock {
            id: "cert-1".into(),
            name: "AWS Certified".into(),
            issuer: Some("Amazon".into()),
            date: Some("2024".into()),
        }],
        languages: vec![ResumeLanguageBlock {
            id: "lang-1".into(),
            name: "Français".into(),
            level: "natif".into(),
        }],
    }
}

#[test]
fn export_pdf_reproduit_toutes_les_sections() {
    let resume = build(&document_complet());
    let octets = resume.render_bytes().unwrap();
    let document = lopdf::Document::load_mem(&octets).unwrap();
    assert_eq!(document.get_pages().len(), 1);
    assert_a4_media_box(&document);

    let texte = document.extract_text(&[1]).unwrap();
    let texte = texte.to_lowercase();
    for section in [
        "profil",
        "expériences",
        "projets",
        "compétences",
        "formation",
        "certifications",
        "langues",
    ] {
        assert!(
            texte.contains(section),
            "section manquante dans le PDF : {section}"
        );
    }
    assert!(texte.contains("profil autonome de test"));
    assert!(texte.contains("mission principale"));
    assert!(texte.contains("atlas"));
    assert!(texte.contains("rust"));
    assert!(texte.contains("aws certified"));
    assert!(texte.contains("français"));
}
