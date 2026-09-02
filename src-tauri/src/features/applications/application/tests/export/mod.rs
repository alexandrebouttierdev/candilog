//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::applications::domain::{
    ApplicationStatus, ApplicationType, WeeklyWorkSchedule,
};
use crate::features::companies::domain::CompanySize;

/// Candidature de test, dont seuls les champs observés varient.
///
/// La ville n'est renseignée que côté `effective_*` : c'est ce que l'export doit écrire,
/// que la valeur vienne d'une surcharge ou de l'entreprise.
fn cand(job_title: &str, notes: Option<&str>) -> Application {
    Application {
        id: uuid::Uuid::nil(),
        job_title: job_title.into(),
        company_id: uuid::Uuid::nil(),
        company_name: Some("Nova Digital".into()),
        company_size: CompanySize::Pme,
        contact_id: None,
        application_type: ApplicationType::JobOffer,
        contract_type_code: "CDI".into(),
        contract_type_name: Some("CDI".into()),
        weekly_work_schedule: WeeklyWorkSchedule::FullTime,
        weekly_hours: Some(35.0),
        professional_domain_id: Some("M18".into()),
        professional_domain_name: Some("Informatique / Télécommunication".into()),
        city: None,
        address: None,
        company_type_id: None,
        effective_city: Some("Rennes".into()),
        effective_address: Some("12 rue des Lilas".into()),
        effective_company_type_id: Some("IT_SERVICES_COMPANY".into()),
        effective_company_type_name: Some("ESN / Société de services numériques".into()),
        status: ApplicationStatus::Pending,
        sent_date: "2026-08-20".into(),
        job_url: Some("https://example.org/offre".into()),
        notes: notes.map(Into::into),
        created_at: "2026-08-20T00:00:00Z".into(),
        updated_at: "2026-08-20T00:00:00Z".into(),
    }
}

/// Nombre de colonnes de l'export, en-tête comprise.
const COLONNES: usize = 15;

mod le_csv_s_ouvre_correctement_dans_un_tableur;
mod test_entete_precede_les_lignes;
mod test_le_separateur_est_le_point_virgule;
mod test_les_champs_absents_deviennent_vides;
mod test_les_valeurs_effectives_sont_exportees;
mod test_un_champ_contenant_le_separateur_est_echappe;
