//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::applications::domain::{ApplicationType, WeeklyWorkSchedule};
use crate::features::companies::domain::CompanySize;
use std::sync::Mutex;

/// Candidature de test, renvoyée par le dépôt double.
fn cand(job_title: &str, status: ApplicationStatus) -> Application {
    Application {
        id: uuid::Uuid::nil(),
        job_title: job_title.into(),
        company_id: uuid::Uuid::nil(),
        company_name: Some("Nova Digital".into()),
        company_size: CompanySize::Unknown,
        contact_id: None,
        application_type: ApplicationType::JobOffer,
        contract_type_code: "CDI".into(),
        contract_type_name: Some("CDI".into()),
        weekly_work_schedule: WeeklyWorkSchedule::Unspecified,
        weekly_hours: None,
        professional_domain_id: None,
        professional_domain_name: None,
        city: None,
        address: None,
        company_type_id: None,
        effective_city: None,
        effective_address: None,
        effective_company_type_id: None,
        effective_company_type_name: None,
        status,
        sent_date: "2026-08-20".into(),
        job_url: None,
        notes: None,
        created_at: "2026-08-20T00:00:00Z".into(),
        updated_at: "2026-08-20T00:00:00Z".into(),
    }
}

/// Payload utile de test, valide par défaut : réponse à une offre, lien renseigné.
fn new(job_title: &str) -> NewApplication {
    NewApplication {
        job_title: job_title.into(),
        company_id: uuid::Uuid::from_u128(1),
        contact_id: None,
        application_type: ApplicationType::JobOffer,
        contract_type_code: "CDI".into(),
        weekly_work_schedule: WeeklyWorkSchedule::Unspecified,
        weekly_hours: None,
        professional_domain_id: None,
        city: None,
        address: None,
        company_type_id: None,
        status: ApplicationStatus::Pending,
        sent_date: "2026-08-20".into(),
        job_url: Some("https://example.org/offre".into()),
        notes: None,
    }
}

/// Dépôt double : renvoie ce qu'on lui donne, et retient la dernière saisie reçue.
///
/// Retenir la saisie est nécessaire pour observer la **normalisation** du service — sans
/// cela, rien ne distinguerait un lien effacé d'un lien simplement absent du résultat.
#[derive(Default)]
struct StubRepo {
    last_input: Mutex<Option<NewApplication>>,
}

impl StubRepo {
    /// Dernière saisie transmise au dépôt.
    fn recu(&self) -> NewApplication {
        self.last_input.lock().unwrap().clone().unwrap()
    }

    fn retenir(&self, input: &NewApplication) {
        *self.last_input.lock().unwrap() = Some(input.clone());
    }
}

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
        self.retenir(input);
        Ok(cand(&input.job_title, input.status))
    }
    fn update(&self, _id: uuid::Uuid, input: &NewApplication) -> AppResult<Application> {
        self.retenir(input);
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
mod test_creer_refuse_un_contrat_vide;
mod test_creer_refuse_un_lien_d_offre_non_http;
mod test_creer_refuse_une_entreprise_vide;
mod test_creer_valide_delegue_au_depot;
mod test_date_dans_un_autre_format_est_refusee;
mod test_les_heures_hebdomadaires_sont_bornees;
mod test_modifier_valide_les_memes_regles_que_creer;
mod test_une_offre_exige_un_lien_et_une_spontanee_l_efface;
