//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::candidatures::domain::TypeContrat;

/// Candidature de test, renvoyée par le dépôt double.
fn cand(poste: &str, statut: StatutCandidature) -> Candidature {
    Candidature {
        id: uuid::Uuid::nil(),
        poste: poste.into(),
        entreprise_id: uuid::Uuid::nil(),
        entreprise_nom: Some("Nova Digital".into()),
        entreprise_ville: None,
        contact_id: None,
        type_contrat: TypeContrat::Cdi,
        statut,
        date_envoi: "2026-08-20".into(),
        lien_offre: None,
        notes: None,
        created_at: "2026-08-20T00:00:00Z".into(),
        updated_at: "2026-08-20T00:00:00Z".into(),
    }
}

/// Charge utile de test, valide par défaut.
fn nouvelle(poste: &str) -> NouvelleCandidature {
    NouvelleCandidature {
        poste: poste.into(),
        entreprise_id: uuid::Uuid::nil(),
        type_contrat: TypeContrat::Cdi,
        statut: StatutCandidature::EnAttente,
        date_envoi: "2026-08-20".into(),
        lien_offre: None,
        notes: None,
    }
}

/// Dépôt double : renvoie ce qu'on lui donne, sans base.
struct StubRepo;

impl CandidatureRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Candidature>> {
        Ok(vec![])
    }
    fn get(&self, _id: uuid::Uuid) -> AppResult<Candidature> {
        Ok(cand("Développeur", StatutCandidature::EnAttente))
    }
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        _filtre: &FiltreCandidatures,
    ) -> AppResult<Page<Candidature>> {
        Ok(Page::new(vec![], 0, page, page_size))
    }
    fn repartition(&self, _filtre: &FiltreCandidatures) -> AppResult<RepartitionPipeline> {
        Ok(RepartitionPipeline::default())
    }
    fn create(&self, input: &NouvelleCandidature) -> AppResult<Candidature> {
        Ok(cand(&input.poste, input.statut))
    }
    fn update(&self, _id: uuid::Uuid, input: &NouvelleCandidature) -> AppResult<Candidature> {
        Ok(cand(&input.poste, input.statut))
    }
    fn update_statut(&self, _id: uuid::Uuid, statut: StatutCandidature) -> AppResult<Candidature> {
        Ok(cand("Développeur", statut))
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
}

mod test_changer_statut_delegue_au_depot;
mod test_creer_poste_vide_retourne_validation;
mod test_creer_refuse_un_lien_d_offre_non_http;
mod test_creer_valide_delegue_au_depot;
mod test_date_dans_un_autre_format_est_refusee;
mod test_modifier_valide_les_memes_regles_que_creer;
