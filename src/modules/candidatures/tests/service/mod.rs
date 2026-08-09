//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
struct MockRepo {
    created: Mutex<Vec<String>>,
    statuts: Mutex<Vec<StatutCandidature>>,
    deleted: Mutex<Vec<Uuid>>,
}
fn stub(poste: &str, statut: StatutCandidature) -> Candidature {
    Candidature {
        id: Uuid::nil(),
        poste: poste.into(),
        entreprise_id: Uuid::nil(),
        entreprise_nom: Some("ACME".into()),
        contact_id: None,
        type_contrat: TypeContrat::Cdi,
        statut,
        date_envoi: "now".into(),
        lien_offre: None,
        notes: None,
        created_at: "now".into(),
        updated_at: "now".into(),
    }
}
impl CandidatureRepository for MockRepo {
    fn create(&self, input: &NouvelleCandidature) -> AppResult<Candidature> {
        self.created.lock().unwrap().push(input.poste.clone());
        Ok(stub(&input.poste, input.statut))
    }
    fn list(&self) -> AppResult<Vec<Candidature>> {
        Ok(vec![stub("Dev", StatutCandidature::EnAttente)])
    }
    fn update(&self, _id: Uuid, input: &NouvelleCandidature) -> AppResult<Candidature> {
        Ok(stub(&input.poste, input.statut))
    }
    fn update_statut(&self, _id: Uuid, statut: StatutCandidature) -> AppResult<Candidature> {
        self.statuts.lock().unwrap().push(statut);
        Ok(stub("Dev", statut))
    }
    fn delete(&self, id: Uuid) -> AppResult<()> {
        self.deleted.lock().unwrap().push(id);
        Ok(())
    }
}
fn input(poste: &str) -> NouvelleCandidature {
    NouvelleCandidature {
        poste: poste.into(),
        entreprise_id: Uuid::nil(),
        type_contrat: TypeContrat::Cdi,
        statut: StatutCandidature::EnAttente,
        date_envoi: "2026-08-12".into(),
        lien_offre: None,
        notes: None,
    }
}

mod test_changer_statut_delegue_au_depot;
mod test_creer_date_vide_retourne_validation;
mod test_creer_poste_valide_delegue_au_depot;
mod test_creer_poste_vide_retourne_validation;
mod test_supprimer_delegue_au_depot;
