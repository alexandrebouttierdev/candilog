//! Historique des relations sur une base mémoire migrée.

use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use crate::features::relations::application::RelationHistoryService;

const COMPANY: &str = "00000000-0000-4000-8000-00000000c001";
const CONTACT: &str = "00000000-0000-4000-8000-00000000c002";
const OTHER: &str = "00000000-0000-4000-8000-00000000c003";
const APPLICATION: &str = "00000000-0000-4000-8000-00000000a001";

/// Une entreprise, son contact et une candidature passée par l'entretien, relancée une fois.
fn service() -> (
    RelationHistoryService<SqliteRelationHistoryRepository>,
    SqlitePool,
) {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    connection(&pool)
        .unwrap()
        .execute_batch(&format!(
            "INSERT INTO companies (id, name, company_size, created_at, updated_at)
                 VALUES ('{COMPANY}', 'Vallis Conseil', 'PME', '2026-08-01T08:00:00+00:00', '2026-08-01');
             INSERT INTO companies (id, name, company_size, created_at, updated_at)
                 VALUES ('{OTHER}', 'Autre', 'PME', '2026-08-01T08:00:00+00:00', '2026-08-01');
             INSERT INTO contacts (id, company_id, first_name, name, created_at, updated_at)
                 VALUES ('{CONTACT}', '{COMPANY}', 'Claire', 'Ménard', '2026-08-02T08:00:00+00:00', '2026-08-02');
             INSERT INTO applications
                 (id, company_id, contact_id, job_title, contract_type_code, status, sent_date, created_at, updated_at)
                 VALUES ('{APPLICATION}', '{COMPANY}', '{CONTACT}', 'Technicien support N2', 'CDI',
                         'ENTRETIEN', '2026-08-29', '2026-08-29T09:00:00+00:00', '2026-08-29');
             INSERT INTO status_history (id, application_id, status, changed_at)
                 VALUES ('h1', '{APPLICATION}', 'EN_ATTENTE', '2026-08-29T09:00:00+00:00'),
                        ('h2', '{APPLICATION}', 'ENTRETIEN', '2026-09-02T10:00:00+00:00');
             INSERT INTO follow_ups (id, application_id, follow_up_date, type, created_at, done_at)
                 VALUES ('f1', '{APPLICATION}', '2026-09-01', 'Email', '2026-08-29', '2026-09-01T07:00:00+00:00'),
                        ('f2', '{APPLICATION}', '2026-09-20', 'Email', '2026-08-29', NULL);
             INSERT INTO interviews (id, application_id, interview_date, type, created_at, updated_at)
                 VALUES ('i1', '{APPLICATION}', '2026-09-12T14:30:00+02:00', 'Visio', '2026-09-02', '2026-09-02');"
        ))
        .unwrap();
    (
        RelationHistoryService::new(SqliteRelationHistoryRepository::new(pool.clone())),
        pool,
    )
}

fn company() -> RelationRef {
    RelationRef {
        kind: RelationKind::Company,
        id: Uuid::parse_str(COMPANY).unwrap(),
    }
}

fn contact() -> RelationRef {
    RelationRef {
        kind: RelationKind::Contact,
        id: Uuid::parse_str(CONTACT).unwrap(),
    }
}

fn note(relation: RelationRef, body: &str, noted_on: &str) -> NewRelationNote {
    NewRelationNote {
        relation,
        body: body.into(),
        noted_on: noted_on.into(),
    }
}

#[test]
fn l_historique_ne_reprend_que_des_faits_enregistres_du_plus_recent_au_plus_ancien() {
    let (service, _) = service();
    let kinds: Vec<HistoryKind> = service
        .history(company())
        .unwrap()
        .iter()
        .map(|entry| entry.kind)
        .collect();

    // Le statut initial et la relance encore à faire n'en font pas partie.
    assert_eq!(
        kinds,
        vec![
            HistoryKind::Interview,
            HistoryKind::StatusChanged,
            HistoryKind::FollowUpDone,
            HistoryKind::ApplicationSent,
            HistoryKind::Added,
        ]
    );
    let interview = &service.history(company()).unwrap()[0];
    assert_eq!(interview.detail.as_deref(), Some("Visio"));
    assert_eq!(
        interview.job_title.as_deref(),
        Some("Technicien support N2")
    );
}

#[test]
fn une_note_s_ajoute_a_sa_fiche_seulement_et_se_supprime() {
    let (service, _) = service();
    let saved = service
        .add_note(&note(
            contact(),
            "  Réponse positive de Claire  ",
            "2026-09-02",
        ))
        .unwrap();
    assert_eq!(saved.body, "Réponse positive de Claire");

    let contact_history = service.history(contact()).unwrap();
    let entry = contact_history
        .iter()
        .find(|entry| entry.kind == HistoryKind::Note)
        .unwrap();
    assert_eq!(entry.note_id, Some(saved.id));
    assert_eq!(entry.at, "2026-09-02");
    assert!(!service
        .history(company())
        .unwrap()
        .iter()
        .any(|entry| entry.kind == HistoryKind::Note));

    service.delete_note(saved.id).unwrap();
    assert!(matches!(
        service.delete_note(saved.id),
        Err(AppError::NotFound(_))
    ));
}

#[test]
fn une_note_vide_trop_longue_ou_mal_datee_est_refusee() {
    let (service, _) = service();
    for invalid in [
        note(company(), "   ", "2026-09-02"),
        note(company(), &"x".repeat(2001), "2026-09-02"),
        note(company(), "Appel", "02/09/2026"),
    ] {
        assert!(matches!(
            service.add_note(&invalid),
            Err(AppError::Validation(_))
        ));
    }
}

#[test]
fn une_fiche_inconnue_est_signalee() {
    let (service, _) = service();
    let unknown = RelationRef {
        kind: RelationKind::Contact,
        id: Uuid::new_v4(),
    };
    assert!(matches!(
        service.history(unknown),
        Err(AppError::NotFound(_))
    ));
    assert!(matches!(
        service.add_note(&note(unknown, "Appel", "2026-09-02")),
        Err(AppError::NotFound(_))
    ));
}

#[test]
fn les_notes_disparaissent_avec_leur_fiche() {
    let (service, pool) = service();
    let other = RelationRef {
        kind: RelationKind::Company,
        id: Uuid::parse_str(OTHER).unwrap(),
    };
    service
        .add_note(&note(other, "Salon de l'emploi", "2026-09-05"))
        .unwrap();
    let conn = connection(&pool).unwrap();
    conn.execute("DELETE FROM companies WHERE id = ?1", [OTHER])
        .unwrap();
    let left: i64 = conn
        .query_row("SELECT count(*) FROM relation_notes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(left, 0);
}
