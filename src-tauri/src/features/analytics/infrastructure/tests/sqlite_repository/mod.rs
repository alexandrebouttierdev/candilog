//! Vérification des agrégats `SQLite` sur une base mémoire migrée.

use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use uuid::Uuid;

fn context() -> (SqliteAnalyticsRepository, Uuid) {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let company = Uuid::new_v4();
    connection(&pool)
        .unwrap()
        .execute(
            "INSERT INTO companies (id, name, city, created_at, updated_at)
             VALUES (?1, 'Nova Digital', 'Rennes', '2026-01-01', '2026-01-01')",
            [company.to_string()],
        )
        .unwrap();
    (SqliteAnalyticsRepository::new(pool), company)
}

fn application(
    repo: &SqliteAnalyticsRepository,
    company: Uuid,
    status: &str,
    date: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO applications (
                id, company_id, job_title, contract_type, status, sent_date, created_at, updated_at
             ) VALUES (?1, ?2, 'Développeur Rust', 'CDI', ?3, ?4, ?4, ?4)",
            rusqlite::params![id.to_string(), company.to_string(), status, date],
        )
        .unwrap();
    id
}

#[test]
fn indicateurs_conservent_les_etapes_atteintes_apres_un_refus() {
    let (repo, company) = context();
    let refusee = application(&repo, company, "REFUS", "2026-08-10");
    application(&repo, company, "EN_ATTENTE", "2026-08-12");
    application(&repo, company, "RELANCEE", "2026-06-01");
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO status_history (id, application_id, status, changed_at)
             VALUES (?1, ?2, 'ENTRETIEN', '2026-08-18')",
            rusqlite::params![Uuid::new_v4().to_string(), refusee.to_string()],
        )
        .unwrap();

    let metrics = repo.metrics(Some("2026-08-01")).unwrap();

    assert_eq!(metrics.applications, 2);
    assert_eq!(metrics.interviews, 1);
    assert_eq!(metrics.responses, 1);
    assert_eq!(metrics.rejected, 1);
    assert_eq!(metrics.pending, 1);
    assert_eq!(metrics.response_rate, 50);
}

#[test]
fn activite_retourne_toutes_les_semaines_meme_vides() {
    let (repo, company) = context();
    let aujourd_hui = chrono::Local::now().date_naive();
    let cette_week = (aujourd_hui - chrono::Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    let week_previous = (aujourd_hui - chrono::Duration::days(9))
        .format("%Y-%m-%d")
        .to_string();
    application(&repo, company, "EN_ATTENTE", &cette_week);
    application(&repo, company, "EN_ATTENTE", &week_previous);

    let activity = repo.activity_hebdomadaire(4).unwrap();

    assert_eq!(activity.len(), 4);
    assert_eq!(activity[2].count, 1);
    assert_eq!(activity[3].count, 1);
}

#[test]
fn upcoming_items_ne_retiennent_que_le_futur_et_restent_ordonnees() {
    let (repo, company) = context();
    let application = application(&repo, company, "EN_ATTENTE", "2026-08-01");
    let conn = connection(&repo.pool).unwrap();
    for (date, channel) in [("2026-08-20", "Email"), ("2026-09-02", "Téléphone")] {
        conn.execute(
            "INSERT INTO follow_ups (id, application_id, follow_up_date, type, created_at)
             VALUES (?1, ?2, ?3, ?4, ?3)",
            rusqlite::params![
                Uuid::new_v4().to_string(),
                application.to_string(),
                date,
                channel
            ],
        )
        .unwrap();
    }
    conn.execute(
        "INSERT INTO interviews (
            id, application_id, interview_date, type, created_at, updated_at
         ) VALUES (?1, ?2, '2026-09-01T14:00:00+02:00', 'Visio', '2026-08-01', '2026-08-01')",
        rusqlite::params![Uuid::new_v4().to_string(), application.to_string()],
    )
    .unwrap();

    let upcoming_items = repo.upcoming_items("2026-08-28", 5).unwrap();

    assert_eq!(upcoming_items.len(), 2);
    assert_eq!(upcoming_items[0].kind, "entretien");
    assert_eq!(upcoming_items[1].kind, "relance");
}

#[test]
fn candidatures_to_follow_up_respectent_age_statut_et_limite() {
    let (repo, company) = context();
    application(&repo, company, "EN_ATTENTE", "2026-08-10");
    application(&repo, company, "EN_ATTENTE", "2026-08-25");
    application(&repo, company, "REFUS", "2026-08-01");

    let items = repo.to_follow_up("2026-08-28", 7, 1).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].days, 18);
}

#[test]
fn recentes_restituent_les_jointures_et_les_enums_du_domaine() {
    let (repo, company) = context();
    application(&repo, company, "ENTRETIEN", "2026-08-20");

    let items = repo.recent(3).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].company_name.as_deref(), Some("Nova Digital"));
    assert_eq!(items[0].status, ApplicationStatus::Interview);
    assert_eq!(items[0].contract_type, ContractType::Cdi);
}
