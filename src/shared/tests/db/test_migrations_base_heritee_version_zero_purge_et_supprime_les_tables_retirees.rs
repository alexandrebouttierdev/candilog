//! Cas de test isolé.

use super::*;

#[test]
fn test_migrations_base_heritee_version_zero_purge_et_supprime_les_tables_retirees() {
    // Contrairement au test précédent (base déjà à jour), celui-ci part d'une base
    // à `user_version = 0` : les trois migrations s'exécutent réellement, ce qui
    // vérifie le comportement métier de 002 (purge ciblée) et 003 (DROP TABLE),
    // pas seulement leur non-rejeu.
    let pool = open_pool(None).unwrap();
    {
        let conn = pool.get().unwrap();
        conn.execute_batch(
            "CREATE TABLE offres (url TEXT PRIMARY KEY);
                 CREATE TABLE local_meta (cle TEXT PRIMARY KEY, valeur TEXT NOT NULL);
                 CREATE TABLE llm_appels (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    operation TEXT NOT NULL, provider TEXT NOT NULL, modele TEXT NOT NULL,
                    latence_ms INTEGER NOT NULL, succes INTEGER NOT NULL, cree_le TEXT NOT NULL);
                 INSERT INTO llm_appels (operation, provider, modele, latence_ms, succes, cree_le)
                    VALUES ('score_offre', 'ollama', 'm', 1, 1, '2026-01-01T00:00:00Z');
                 INSERT INTO llm_appels (operation, provider, modele, latence_ms, succes, cree_le)
                    VALUES ('generate_cv', 'ollama', 'm', 1, 1, '2026-01-01T00:00:01Z');",
        )
        .unwrap();
        // `user_version` reste à 0 (défaut) : simule une base héritée jamais migrée.
    }
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    for table in ["offres", "local_meta"] {
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 0, "table {table} n'a pas été supprimée");
    }
    let score_offre: i64 = conn
        .query_row(
            "SELECT count(*) FROM llm_appels WHERE operation='score_offre'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        score_offre, 0,
        "les lignes score_offre n'ont pas été purgées"
    );
    let generate_cv: i64 = conn
        .query_row(
            "SELECT count(*) FROM llm_appels WHERE operation='generate_cv'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        generate_cv, 1,
        "la purge a supprimé une opération qu'elle ne devait pas toucher"
    );
}
