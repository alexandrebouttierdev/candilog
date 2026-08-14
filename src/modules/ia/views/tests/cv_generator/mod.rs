//! Helpers communs et déclaration des cas de test.
use super::workbench::{missing_skills, panel_footer_state, present_skills, PanelFooterState};
use super::workflow::detected_company;
use crate::modules::entreprises::model::Entreprise;
use crate::modules::ia::cv_model::MatchScore;

fn entreprise(nom: &str) -> Entreprise {
    Entreprise {
        id: uuid::Uuid::nil(),
        nom: nom.to_owned(),
        secteur: None,
        type_: None,
        site_web: None,
        ville: None,
        adresse: None,
        notes: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn score(matched: &[&str], missing: &[&str]) -> MatchScore {
    MatchScore {
        total: 0,
        skills: 0,
        experience: 0,
        ats: 0,
        matched: matched.iter().map(|skill| (*skill).to_owned()).collect(),
        missing: missing.iter().map(|skill| (*skill).to_owned()).collect(),
    }
}

fn owned(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

mod analyse_seule_propose_la_generation;
mod aucune_entreprise_connue_ne_detecte_rien;
mod detecte_une_entreprise_citee_dans_l_offre;
mod generation_en_cours_affiche_la_progression;
mod generation_terminee_laisse_place_aux_suggestions;
mod l_entreprise_la_plus_tot_est_retournee;
mod l_ordre_d_apparition_est_preserve;
mod la_casse_est_ignoree;
mod la_detection_ignore_la_casse;
mod la_detection_ne_reconnait_pas_un_nom_court_dans_un_mot;
mod le_nom_compose_prime_sur_un_sous_nom;
mod les_doublons_sont_supprimes;
mod regeneration_conserve_les_suggestions_affichees;
mod sans_analyse_le_pied_est_vide;
mod un_texte_vide_ne_detecte_rien;
mod une_liste_vide_reste_vide;
