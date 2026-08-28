//! Cas de test isolé.

use super::*;

#[test]
fn test_suppression_candidature_efface_ses_relances_en_cascade() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    conn.execute_batch(
            "INSERT INTO entreprises (id, nom, created_at, updated_at)
                VALUES ('e1', 'ACME', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
             INSERT INTO candidatures (id, entreprise_id, poste, date_envoi, created_at, updated_at)
                VALUES ('c1', 'e1', 'Dev', '2026-01-01', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
             INSERT INTO relances (id, candidature_id, date_relance, created_at)
                VALUES ('r1', 'c1', '2026-01-05', '2026-01-05T00:00:00Z');
             DELETE FROM candidatures WHERE id = 'c1';",
        )
        .unwrap();
    let restantes: i64 = conn
        .query_row("SELECT count(*) FROM relances", [], |r| r.get(0))
        .unwrap();
    assert_eq!(restantes, 0);
}
