//! Helpers communs et déclaration des cas de test.
use super::{entretiens_planifies, total_candidatures_liees};
use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
use crate::modules::entretiens::model::{Entretien, TypeEntretien};
use uuid::Uuid;

fn candidature(contact_id: Option<Uuid>) -> Candidature {
    Candidature {
        id: Uuid::new_v4(),
        poste: "Développeur".into(),
        entreprise_id: Uuid::new_v4(),
        entreprise_nom: Some("Agrial".into()),
        contact_id,
        type_contrat: TypeContrat::Cdi,
        statut: StatutCandidature::EnAttente,
        date_envoi: "2026-08-01".into(),
        lien_offre: None,
        notes: None,
        created_at: "2026-08-01".into(),
        updated_at: "2026-08-01".into(),
    }
}

fn entretien(contact_id: Option<Uuid>, date: &str) -> Entretien {
    Entretien {
        id: Uuid::new_v4(),
        candidature_id: Uuid::new_v4(),
        contact_id,
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

mod les_candidatures_liees_comptent_toutes_celles_qui_ont_un_contact;
mod les_entretiens_planifies_sont_globaux_a_partir_d_aujourd_hui;
