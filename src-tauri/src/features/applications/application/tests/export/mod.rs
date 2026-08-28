//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::applications::domain::{ApplicationStatus, ContractType};

/// Candidature de test, dont seuls les champs observés varient.
fn cand(job_title: &str, notes: Option<&str>) -> Application {
    Application {
        id: uuid::Uuid::nil(),
        job_title: job_title.into(),
        company_id: uuid::Uuid::nil(),
        company_name: Some("Nova Digital".into()),
        company_city: Some("Rennes".into()),
        contact_id: None,
        contract_type: ContractType::Cdi,
        status: ApplicationStatus::Pending,
        sent_date: "2026-08-20".into(),
        job_url: None,
        notes: notes.map(Into::into),
        created_at: "2026-08-20T00:00:00Z".into(),
        updated_at: "2026-08-20T00:00:00Z".into(),
    }
}

mod test_entete_precede_les_lignes;
mod test_le_separateur_est_le_point_virgule;
mod test_les_champs_absents_deviennent_vides;
mod test_un_champ_contenant_le_separateur_est_echappe;
