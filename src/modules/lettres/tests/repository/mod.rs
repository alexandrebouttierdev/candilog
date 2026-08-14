//! Helpers et déclaration des tests du dépôt de lettres.

use super::*;
use crate::modules::lettres::dtos::NouvelleLettre;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteLettreRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteLettreRepository::new(pool)
}

fn letter(name: &str) -> NouvelleLettre {
    NouvelleLettre {
        name: name.into(),
        company: Some("Candilog".into()),
        job_title: Some("Développeur Rust".into()),
        tone: "formal".into(),
        length: "medium".into(),
        content: "Madame, Monsieur, voici ma candidature.".into(),
    }
}

mod enregistrer_et_recharger_restitue_le_document;
mod supprimer_retire_la_lettre_de_la_bibliotheque;
