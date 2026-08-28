//! Helpers communs et déclaration des cas de test.
use super::*;

/// FollowUp de test, renvoyée par le dépôt double.
fn rel(date: &str, channel: &str) -> FollowUp {
    FollowUp {
        id: uuid::Uuid::nil(),
        application_id: uuid::Uuid::from_u128(1),
        application_job_title: Some("Développeur Frontend".into()),
        company_name: Some("Nova Digital".into()),
        follow_up_date: date.into(),
        channel: channel.into(),
        notes: None,
        created_at: "2026-08-20T00:00:00Z".into(),
    }
}

/// Payload utile valide par défaut.
fn new(date: &str) -> NewFollowUp {
    NewFollowUp {
        application_id: uuid::Uuid::from_u128(1),
        follow_up_date: date.into(),
        channel: "Email".into(),
        notes: None,
    }
}

/// Dépôt double : renvoie ce qu'on lui donne, sans base.
struct StubRepo;

impl FollowUpRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<FollowUp>> {
        Ok(vec![])
    }
    fn list_between(&self, _from: &str, _to: &str) -> AppResult<Vec<FollowUp>> {
        Ok(vec![])
    }
    fn create(&self, input: &NewFollowUp) -> AppResult<FollowUp> {
        Ok(rel(&input.follow_up_date, &input.channel))
    }
    fn update(&self, _id: uuid::Uuid, input: &NewFollowUp) -> AppResult<FollowUp> {
        Ok(rel(&input.follow_up_date, &input.channel))
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
}

mod test_canal_vide_est_refuse;
mod test_creer_valide_delegue_au_depot;
mod test_date_avec_heure_est_refusee;
