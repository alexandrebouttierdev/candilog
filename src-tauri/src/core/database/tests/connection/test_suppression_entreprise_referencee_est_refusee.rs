//! Cas de test isolé.

use super::*;

#[test]
fn test_suppression_entreprise_referencee_est_refusee() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    conn.execute_batch(
            "INSERT INTO companies (id, name, created_at, updated_at)
                VALUES ('e1', 'ACME', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
             INSERT INTO applications (id, company_id, job_title, sent_date, created_at, updated_at)
                VALUES ('c1', 'e1', 'Dev', '2026-01-01', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');",
        )
        .unwrap();
    let error = conn.execute("DELETE FROM companies WHERE id = 'e1'", []);
    assert!(error.is_err(), "RESTRICT non appliqué");
}
