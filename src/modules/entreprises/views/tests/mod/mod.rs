//! Helpers communs et déclaration des cas de test.
use super::total_candidatures;
use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
use crate::modules::entreprises::model::Entreprise;
use uuid::Uuid;

fn candidature(entreprise_id: Uuid) -> Candidature {
    Candidature {
        id: Uuid::new_v4(),
        poste: "Développeur".into(),
        entreprise_id,
        entreprise_nom: Some("Agrial".into()),
        contact_id: None,
        type_contrat: TypeContrat::Cdi,
        statut: StatutCandidature::EnAttente,
        date_envoi: "2026-08-01".into(),
        lien_offre: None,
        notes: None,
        created_at: "2026-08-01".into(),
        updated_at: "2026-08-01".into(),
    }
}

fn entreprise(id: Uuid) -> Entreprise {
    Entreprise {
        id,
        nom: "Agrial".into(),
        secteur: None,
        type_: None,
        site_web: None,
        ville: None,
        adresse: None,
        notes: None,
        created_at: "2026-08-01".into(),
        updated_at: "2026-08-01".into(),
    }
}

mod le_compteur_est_global_et_ne_depend_pas_de_la_selection;
mod ne_comptent_que_les_candidatures_rattachees_a_une_entreprise_suivie;
mod sans_entreprise_aucune_candidature_n_est_comptee;
