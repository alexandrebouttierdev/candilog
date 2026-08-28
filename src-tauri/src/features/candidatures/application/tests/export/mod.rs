//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::candidatures::domain::{StatutCandidature, TypeContrat};

/// Candidature de test, dont seuls les champs observés varient.
fn cand(poste: &str, notes: Option<&str>) -> Candidature {
    Candidature {
        id: uuid::Uuid::nil(),
        poste: poste.into(),
        entreprise_id: uuid::Uuid::nil(),
        entreprise_nom: Some("Nova Digital".into()),
        entreprise_ville: Some("Rennes".into()),
        contact_id: None,
        type_contrat: TypeContrat::Cdi,
        statut: StatutCandidature::EnAttente,
        date_envoi: "2026-08-20".into(),
        lien_offre: None,
        notes: notes.map(Into::into),
        created_at: "2026-08-20T00:00:00Z".into(),
        updated_at: "2026-08-20T00:00:00Z".into(),
    }
}

mod test_entete_precede_les_lignes;
mod test_le_separateur_est_le_point_virgule;
mod test_les_champs_absents_deviennent_vides;
mod test_un_champ_contenant_le_separateur_est_echappe;
