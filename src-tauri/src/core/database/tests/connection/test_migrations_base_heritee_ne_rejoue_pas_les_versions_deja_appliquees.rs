//! Cas de test isolé.

use super::*;

#[test]
fn test_migrations_base_heritee_ne_rejoue_pas_les_versions_deja_appliquees() {
    let pool = open_pool(None).unwrap();
    {
        let conn = pool.get().unwrap();
        // Base héritée : marquée en version 3, avec une ligne que la migration 2
        // supprimerait si elle était rejouée.
        conn.execute_batch(
            "CREATE TABLE llm_appels (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    operation TEXT NOT NULL, provider TEXT NOT NULL, modele TEXT NOT NULL,
                    latence_ms INTEGER NOT NULL, succes INTEGER NOT NULL, cree_le TEXT NOT NULL);
                 INSERT INTO llm_appels (operation, provider, modele, latence_ms, succes, cree_le)
                    VALUES ('score_offre', 'ollama', 'm', 1, 1, '2026-01-01T00:00:00Z');",
        )
        .unwrap();
        conn.pragma_update(None, "user_version", 3_i64).unwrap();
    }
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    let restantes: i64 = conn
        .query_row(
            "SELECT count(*) FROM llm_appels WHERE operation='score_offre'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(restantes, 1, "une migration déjà appliquée a été rejouée");
}
