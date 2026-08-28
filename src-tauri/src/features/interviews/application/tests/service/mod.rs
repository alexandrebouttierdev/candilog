//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::interviews::domain::InterviewType;

/// Interview de test, renvoyé par le dépôt double.
fn ent(date: &str) -> Interview {
    Interview {
        id: uuid::Uuid::nil(),
        application_id: uuid::Uuid::from_u128(1),
        application_job_title: Some("Développeur Frontend".into()),
        company_name: Some("Nova Digital".into()),
        contact_id: None,
        contact_name: None,
        interview_date: date.into(),
        type_interview: InterviewType::Video,
        location: None,
        notes: None,
        minutes: None,
        analysis_ai: None,
        created_at: "2026-08-20T00:00:00Z".into(),
        updated_at: "2026-08-20T00:00:00Z".into(),
    }
}

/// Payload utile valide par défaut.
fn new(date: &str) -> NewInterview {
    NewInterview {
        application_id: uuid::Uuid::from_u128(1),
        contact_id: None,
        interview_date: date.into(),
        type_interview: InterviewType::Video,
        location: None,
        notes: None,
        minutes: None,
    }
}

/// Dépôt double : renvoie ce qu'on lui donne, sans base.
struct StubRepo;

impl InterviewRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Interview>> {
        Ok(vec![])
    }
    fn list_between(&self, _from: &str, _to: &str) -> AppResult<Vec<Interview>> {
        Ok(vec![])
    }
    fn get(&self, _id: uuid::Uuid) -> AppResult<Interview> {
        Ok(ent("2026-08-25T14:00:00+02:00"))
    }
    fn save_and_mark_candidate(
        &self,
        _id: Option<uuid::Uuid>,
        input: &NewInterview,
    ) -> AppResult<Interview> {
        Ok(ent(&input.interview_date))
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
    fn save_analysis(&self, _id: uuid::Uuid, _analysis: &InterviewAnalysis) -> AppResult<()> {
        Ok(())
    }
}

mod test_candidature_nulle_est_refusee;
mod test_date_sans_heure_est_refusee;
mod test_enregistrer_valide_delegue_au_depot;
