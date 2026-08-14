//! Helpers communs et déclaration des cas de test.
use super::{month_counts, week_start};
use crate::modules::entretiens::model::{Entretien, TypeEntretien};
use crate::modules::relances::model::Relance;
use chrono::{Datelike, NaiveDate};
use uuid::Uuid;

fn entretien(date: &str) -> Entretien {
    Entretien {
        id: Uuid::new_v4(),
        candidature_id: Uuid::new_v4(),
        contact_id: None,
        date_entretien: date.into(),
        type_entretien: TypeEntretien::Visio,
        lieu: None,
        notes: None,
        compte_rendu: None,
        analyse_ia: None,
        created_at: date.into(),
        updated_at: date.into(),
    }
}

fn relance(date: &str) -> Relance {
    Relance {
        id: Uuid::new_v4(),
        candidature_id: Uuid::new_v4(),
        date_relance: date.into(),
        type_relance: "Email".into(),
        notes: None,
        created_at: date.into(),
    }
}

mod la_semaine_commence_toujours_un_lundi;
mod le_compteur_accepte_les_horodatages_complets;
mod le_compteur_ne_reitent_que_le_mois_affiche;
