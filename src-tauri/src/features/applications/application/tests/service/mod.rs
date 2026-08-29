//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::applications::domain::ContractType;

/// Candidature de test, renvoyée par le dépôt double.
fn cand(job_title: &str, status: ApplicationStatus) -> Application {
    Application {
        id: uuid::Uuid::nil(),
        job_title: job_title.into(),
        company_id: uuid::Uuid::nil(),
        company_name: Some("Nova Digital".into()),
        company_city: None,
        contact_id: None,
        contract_type: ContractType::Cdi,
        status,
        sent_date: "2026-08-20".into(),
        job_url: None,
        notes: None,
        created_at: "2026-08-20T00:00:00Z".into(),
        updated_at: "2026-08-20T00:00:00Z".into(),
    }
}

/// Payload utile de test, valide par défaut.
fn new(job_title: &str) -> NewApplication {
    NewApplication {
        job_title: job_title.into(),
        company_id: uuid::Uuid::from_u128(1),
        contract_type: ContractType::Cdi,
        status: ApplicationStatus::Pending,
        sent_date: "2026-08-20".into(),
        job_url: None,
        notes: None,
    }
}

/// Dépôt double : renvoie ce qu'on lui donne, sans base.
struct StubRepo;

impl ApplicationRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Application>> {
        Ok(vec![])
    }
    fn get(&self, _id: uuid::Uuid) -> AppResult<Application> {
        Ok(cand("Développeur", ApplicationStatus::Pending))
    }
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        _filter: &ApplicationFilter,
    ) -> AppResult<Page<Application>> {
        Ok(Page::new(vec![], 0, page, page_size))
    }
    fn breakdown(&self, _filter: &ApplicationFilter) -> AppResult<PipelineBreakdown> {
        Ok(PipelineBreakdown::default())
    }
    fn create(&self, input: &NewApplication) -> AppResult<Application> {
        Ok(cand(&input.job_title, input.status))
    }
    fn update(&self, _id: uuid::Uuid, input: &NewApplication) -> AppResult<Application> {
        Ok(cand(&input.job_title, input.status))
    }
    fn update_status(&self, _id: uuid::Uuid, status: ApplicationStatus) -> AppResult<Application> {
        Ok(cand("Développeur", status))
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
}

mod test_changer_statut_delegue_au_depot;
mod test_creer_poste_vide_retourne_validation;
mod test_creer_refuse_un_lien_d_offre_non_http;
mod test_creer_refuse_une_entreprise_vide;
mod test_creer_valide_delegue_au_depot;
mod test_date_dans_un_autre_format_est_refusee;
mod test_modifier_valide_les_memes_regles_que_creer;
