//! Cas de test isolé.

use super::*;

#[test]
fn test_migrations_base_heritee_ne_rejoue_pas_les_versions_deja_appliquees() {
    let pool = open_pool(None).unwrap();
    {
        let conn = pool.get().unwrap();
        // Base déjà au schéma courant : `init_schema` ne doit pas être rejoué.
        conn.execute_batch(
            "CREATE TABLE llm_calls (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    operation TEXT NOT NULL, provider TEXT NOT NULL, model TEXT NOT NULL,
                    latency_ms INTEGER NOT NULL, success INTEGER NOT NULL, created_at TEXT NOT NULL);
                 INSERT INTO llm_calls (operation, provider, model, latency_ms, success, created_at)
                    VALUES ('score_offre', 'ollama', 'm', 1, 1, '2026-01-01T00:00:00Z');",
        )
        .unwrap();
        conn.pragma_update(None, "user_version", DERNIERE_VERSION)
            .unwrap();
    }
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    let restantes: i64 = conn
        .query_row(
            "SELECT count(*) FROM llm_calls WHERE operation='score_offre'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(restantes, 1, "une migration déjà appliquée a été rejouée");
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, DERNIERE_VERSION);
}
