//! Vérification des agrégats `SQLite` sur une base mémoire migrée.

use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use crate::features::applications::domain::{ApplicationChannel, ApplicationStatus};
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

fn application(repo: &SqliteAnalyticsRepository, company: Uuid, status: &str, date: &str) -> Uuid {
    let id = Uuid::new_v4();
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO applications (
                id, company_id, job_title, contract_type_code, status, sent_date,
                created_at, updated_at
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
    assert_eq!(items[0].contract_type_code, "CDI");
    assert_eq!(items[0].contract_type_name.as_deref(), Some("CDI"));
    // Les valeurs héritées de l'entreprise sont résolues comme dans le suivi.
    assert_eq!(items[0].effective_city.as_deref(), Some("Rennes"));
}

fn relance(repo: &SqliteAnalyticsRepository, application: Uuid, date: &str, faite: bool) {
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO follow_ups (id, application_id, follow_up_date, created_at, done_at)
             VALUES (?1, ?2, ?3, '2026-01-01', ?4)",
            rusqlite::params![
                Uuid::new_v4().to_string(),
                application.to_string(),
                date,
                faite.then_some("2026-09-01T10:00:00Z")
            ],
        )
        .unwrap();
}

fn entretien(repo: &SqliteAnalyticsRepository, application: Uuid, at: &str) {
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO interviews (id, application_id, interview_date, type, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'Visio', '2026-01-01', '2026-01-01')",
            rusqlite::params![Uuid::new_v4().to_string(), application.to_string(), at],
        )
        .unwrap();
}

#[test]
fn l_agenda_garde_les_retards_et_ecarte_ce_qui_est_fait_ou_passe() {
    let (repo, company) = context();
    let candidature = application(&repo, company, "RELANCEE", "2026-08-01");
    relance(&repo, candidature, "2026-08-20", false); // en retard : reste
    relance(&repo, candidature, "2026-08-25", true); // faite : écartée
    relance(&repo, candidature, "2026-09-25", false); // au-delà de la semaine : écartée
    entretien(&repo, candidature, "2026-09-10T14:30:00"); // passé : écarté
    entretien(&repo, candidature, "2026-09-12T14:30:00"); // aujourd'hui : reste

    let agenda = repo.agenda("2026-09-12", "2026-09-19").unwrap();

    let dates: Vec<_> = agenda.iter().map(|item| item.date.as_str()).collect();
    assert_eq!(dates, vec!["2026-08-20", "2026-09-12T14:30:00"]);
    assert_eq!(agenda[0].kind, AgendaKind::FollowUp);
    assert_eq!(agenda[1].kind, AgendaKind::Interview);
    assert_eq!(agenda[1].detail, "Visio");
    assert_eq!(agenda[0].reference_number, 1);
    assert_eq!(agenda[0].status, ApplicationStatus::FollowedUp);
}

#[test]
fn une_relance_faite_ne_compte_plus_en_retard() {
    let (repo, company) = context();
    let candidature = application(&repo, company, "EN_ATTENTE", "2026-08-01");
    relance(&repo, candidature, "2026-08-20", true);
    relance(&repo, candidature, "2026-08-21", false);

    assert_eq!(repo.performance(None).unwrap().overdue_follow_ups, 1);
}

#[test]
fn le_taux_par_canal_compte_les_reponses_comme_les_metriques() {
    let (repo, company) = context();
    let reseau = application(&repo, company, "ENTRETIEN", "2026-09-01");
    let offre = application(&repo, company, "EN_ATTENTE", "2026-09-02");
    application(&repo, company, "REFUS", "2026-09-03");
    let conn = connection(&repo.pool).unwrap();
    conn.execute(
        "UPDATE applications SET channel = 'NETWORK' WHERE id = ?1",
        [reseau.to_string()],
    )
    .unwrap();
    // Une candidature revenue en attente après un refus a quand même reçu une réponse.
    conn.execute(
        "INSERT INTO status_history (id, application_id, status, changed_at)
         VALUES ('h-refus', ?1, 'REFUS', '2026-09-05')",
        [offre.to_string()],
    )
    .unwrap();

    let taux = repo.channel_rates(None).unwrap();
    let offre = taux
        .iter()
        .find(|rate| rate.channel == ApplicationChannel::Offer)
        .unwrap();
    assert_eq!((offre.applications, offre.responses), (2, 2));
    let reseau = taux
        .iter()
        .find(|rate| rate.channel == ApplicationChannel::Network)
        .unwrap();
    assert_eq!((reseau.applications, reseau.responses), (1, 1));
    // Le plus fourni d'abord.
    assert_eq!(taux[0].channel, ApplicationChannel::Offer);
    // La période borne aussi ce calcul.
    assert!(repo.channel_rates(Some("2026-10-01")).unwrap().is_empty());
}
