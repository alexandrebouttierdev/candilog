//! Cas de test isolé.

use super::*;

#[test]
fn test_suppression_candidature_efface_ses_relances_en_cascade() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    conn.execute_batch(
        "INSERT INTO companies (id, name, created_at, updated_at)
                VALUES ('e1', 'ACME', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
             INSERT INTO applications (id, company_id, job_title, contract_type_code, sent_date,
                    created_at, updated_at)
                VALUES ('c1', 'e1', 'Dev', 'CDI', '2026-01-01', '2026-01-01T00:00:00Z',
                    '2026-01-01T00:00:00Z');
             INSERT INTO follow_ups (id, application_id, follow_up_date, created_at)
                VALUES ('r1', 'c1', '2026-01-05', '2026-01-05T00:00:00Z');
             DELETE FROM applications WHERE id = 'c1';",
    )
    .unwrap();
    let restantes: i64 = conn
        .query_row("SELECT count(*) FROM follow_ups", [], |r| r.get(0))
        .unwrap();
    assert_eq!(restantes, 0);
}
