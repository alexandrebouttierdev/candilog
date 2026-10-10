//! Tests de la composition d'un CV depuis le seul profil (« CV de base »).

use crate::features::profile::domain::{Education, Experience, Identity, Language, Profile, Skill};

/// Profil d'essai complet : une expérience, une formation, deux compétences, une langue et
/// une présentation. De quoi exercer chaque section composée par `compose_base_resume`.
pub fn profil_complet() -> Profile {
    Profile {
        identity: Identity {
            first_name: "Alex".into(),
            name: "Martin".into(),
            email: "alex@example.com".into(),
            title: Some("Développeur Rust".into()),
            resume: Some("Dix ans de back-end.".into()),
            ..Default::default()
        },
        experiences: vec![Experience {
            title: "Développeur".into(),
            company: "Acme".into(),
            start_date: "2019-01".into(),
            end_date: Some("2024-06".into()),
            current: false,
            description: Some("Service de paiement.\nMigration Rust.".into()),
            ..Default::default()
        }],
        education: vec![Education {
            degree: "BTS SIO".into(),
            school: "Lycée Diderot".into(),
            ..Default::default()
        }],
        skills: vec![
            Skill {
                name: "Rust".into(),
                ..Default::default()
            },
            Skill {
                name: "SQL".into(),
                ..Default::default()
            },
        ],
        languages: vec![Language {
            name: "Anglais".into(),
            level: "C1".into(),
        }],
        ..Default::default()
    }
}

mod compose_depuis_un_profil_complet;
mod un_profil_vide_est_refuse;
