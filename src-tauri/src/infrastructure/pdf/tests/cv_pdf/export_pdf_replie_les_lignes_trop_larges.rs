//! Un champ plus large que sa colonne doit se replier, jamais faire échouer l'export.
//!
//! Cas réel : un profil de trois expériences, dont l'employeur porte un nom
//! d'administration complet. L'export refusait le CV avec « Le CV ne tient pas sur une page
//! A4 » alors que la page était à moitié vide — l'intitulé, l'entreprise, les coordonnées
//! et les langues étaient posés sur une ligne unique, sans repli.

use super::*;

fn resume_a_lignes_longues() -> ResumePdf {
    ResumePdf {
        name: "Éloïse Nguyên-Beauchêne".into(),
        subtitle: "Technicienne systèmes & réseaux — cœur d'infrastructure".into(),
        email: "eloise.nguyen-beauchene@exemple.fr".into(),
        city: Some("Châlons-en-Champagne".into()),
        phone: Some("06 09 84 27 51".into()),
        linkedin: Some("linkedin.com/in/eloise-nguyen-beauchene-exemple".into()),
        profile: "Cinq ans d'exploitation d'infrastructures systèmes et réseaux.".into(),
        experiences: vec![ResumeExperience {
            title: "Technicienne systèmes & réseaux — pôle « Infrastructures & Sécurité »".into(),
            company: "Communauté d'Agglomération de Châlons-en-Champagne — Direction des Systèmes d'Information".into(),
            location: Some("Châlons-en-Champagne".into()),
            period: "Janv. 2022 — Aujourd’hui".into(),
            bullets: vec!["Exploitation de l'annuaire Active Directory.".into()],
        }],
        education: vec![ResumeEducation {
            degree: "Licence professionnelle « Métiers des réseaux informatiques et télécommunications »".into(),
            school: "Institut Universitaire de Technologie de Reims-Châlons-Charleville".into(),
            location: Some("Châlons-en-Champagne".into()),
            period: "Sept. 2020 — Juin 2021".into(),
            description: None,
        }],
        languages: vec![
            ResumeLanguage { name: "Français".into(), level: "Langue maternelle".into() },
            ResumeLanguage { name: "Anglais".into(), level: "B2 — lu, écrit, parlé".into() },
            ResumeLanguage { name: "Vietnamien".into(), level: "Notions familiales".into() },
        ],
        ..ResumePdf::default()
    }
}

#[test]
fn un_champ_plus_large_que_sa_colonne_ne_fait_pas_echouer_l_export() {
    let octets = resume_a_lignes_longues()
        .render_bytes()
        .expect("un CV court ne doit pas être refusé parce qu'une ligne est trop large");
    let pdf = lopdf::Document::load_mem(&octets).unwrap();
    assert_a4_media_box(&pdf);

    let texte = pdf.extract_text(&[1]).unwrap();
    for attendu in [
        "Direction des Systèmes",
        "Reims-Châlons-Charleville",
        "eloise.nguyen-beauchene@exemple.fr",
        "Notions familiales",
    ] {
        assert!(
            texte.replace('\n', " ").contains(attendu),
            "« {attendu} » absent du PDF : {texte}"
        );
    }
}

/// Le refus doit nommer sa cause : un CV trop large n'est pas un CV trop long, et le
/// raccourcir n'y changerait rien. Une pastille de compétence ne se replie pas — elle est
/// dessinée d'un bloc, fond compris — c'est donc le champ qui peut encore trop s'élargir.
#[test]
fn une_pastille_plus_large_que_sa_colonne_est_refusee_pour_sa_largeur() {
    let mut resume = resume_a_lignes_longues();
    resume.skill_groups = vec![ResumeSkillGroup {
        name: "Compétences".into(),
        items: vec!["Administration ".repeat(20)],
    }];

    assert!(
        matches!(
            resume.render_bytes(),
            Err(AppError::Validation(message)) if message.contains("largeur")
        ),
        "le refus doit nommer la largeur"
    );
}

/// Une adresse électronique démesurée n'a plus de raison d'échouer : elle se replie, comme
/// dans l'aperçu, et le CV s'exporte.
#[test]
fn un_mot_insecable_se_replie_au_lieu_de_sortir_de_la_page() {
    let mut resume = resume_a_lignes_longues();
    resume.email = format!("{}@exemple.fr", "a".repeat(120));

    assert!(
        resume.render_bytes().is_ok(),
        "un mot insécable doit être coupé, pas refusé"
    );
}
