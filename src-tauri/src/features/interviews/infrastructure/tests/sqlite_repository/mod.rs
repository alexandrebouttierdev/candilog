//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use crate::features::interviews::domain::InterviewType;

/// Dépôt sur base mémoire migrée, avec une candidature déjà créée.
fn context() -> (SqliteInterviewRepository, Uuid) {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = connection(&pool).unwrap();
    let company_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO companies (id, name, created_at, updated_at)
         VALUES (?1, 'Nova Digital', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [company_id.to_string()],
    )
    .unwrap();
    let application_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO applications (id, company_id, job_title, contract_type, status, sent_date,
            created_at, updated_at)
         VALUES (?1, ?2, 'Développeur Frontend', 'CDI', 'EN_ATTENTE', '2026-08-20',
            '2026-08-20T00:00:00Z', '2026-08-20T00:00:00Z')",
        [application_id.to_string(), company_id.to_string()],
    )
    .unwrap();
    drop(conn);
    (SqliteInterviewRepository::new(pool), application_id)
}

/// Payload utile valide, dont seule la date varie selon les tests.
fn entree(application_id: Uuid, date: &str) -> NewInterview {
    NewInterview {
        application_id,
        contact_id: None,
        interview_date: date.into(),
        type_interview: InterviewType::Video,
        location: Some("https://meet.example/abc".into()),
        notes: None,
        minutes: None,
    }
}

/// Status courant d'une candidature.
fn status(repo: &SqliteInterviewRepository, application_id: Uuid) -> String {
    connection(&repo.pool)
        .unwrap()
        .query_row(
            "SELECT status FROM applications WHERE id = ?1",
            [application_id.to_string()],
            |row| row.get(0),
        )
        .unwrap()
}

/// Count d'étapes enregistrées dans l'historique de statut.
fn steps(repo: &SqliteInterviewRepository, application_id: Uuid) -> i64 {
    connection(&repo.pool)
        .unwrap()
        .query_row(
            "SELECT count(*) FROM status_history WHERE application_id = ?1 AND status = 'ENTRETIEN'",
            [application_id.to_string()],
            |row| row.get(0),
        )
        .unwrap()
}

mod test_delete_identifiant_inconnu_retourne_not_found;
mod test_enregistrer_deux_fois_n_historise_qu_une_etape;
mod test_enregistrer_fait_avancer_la_candidature;
mod test_enregistrer_sur_candidature_inconnue_retourne_une_phrase_lisible;
mod test_enregistrer_sur_identifiant_inconnu_retourne_not_found;
mod test_la_plage_du_calendrier_inclut_ses_bornes;
mod test_la_suppression_conserve_le_statut_de_la_candidature;
mod test_le_poste_et_l_entreprise_sont_aplatis;
