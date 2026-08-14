//! Helpers communs et déclaration des cas de test.
use super::{
    column_label, contract_short, next_status, previous_status, status_marker, status_tone,
    PIPELINE,
};
use crate::modules::candidatures::model::{StatutCandidature, TypeContrat};
use crate::ui::theme::{Marker, Tone};

mod chaque_colonne_porte_un_libelle_distinct;
mod chaque_statut_porte_un_ton_et_une_forme_propres;
mod l_attente_reste_neutre_pour_ne_pas_saturer_le_pipeline;
mod la_carte_du_pipeline_s_instancie;
mod le_pipeline_est_parcourable_dans_les_deux_sens;
mod le_refus_est_le_seul_statut_barre;
mod les_contrats_ont_une_abreviation_courte;
