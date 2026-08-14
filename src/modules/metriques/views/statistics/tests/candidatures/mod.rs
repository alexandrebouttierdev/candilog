//! Helpers communs et déclaration des cas de test.
use super::{interview_rate, weekly_counts};
use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
use crate::modules::metriques::components::PipelineCounts;
use chrono::NaiveDate;

fn candidature(date: &str) -> Candidature {
    Candidature {
        id: uuid::Uuid::new_v4(),
        poste: "Développeur".into(),
        entreprise_id: uuid::Uuid::new_v4(),
        entreprise_nom: Some("Agrial".into()),
        contact_id: None,
        type_contrat: TypeContrat::Cdi,
        statut: StatutCandidature::EnAttente,
        date_envoi: date.into(),
        lien_offre: None,
        notes: None,
        created_at: date.into(),
        updated_at: date.into(),
    }
}

mod le_taux_d_entretien_est_un_pourcentage_arrondi_sans_division_par_zero;
mod les_bornes_de_la_fenetre_des_huit_semaines_sont_respectees;
mod les_huit_semaines_s_ordonnent_de_la_plus_ancienne_a_la_courante;
mod un_horodatage_complet_est_compte_par_son_prefixe_de_date;
