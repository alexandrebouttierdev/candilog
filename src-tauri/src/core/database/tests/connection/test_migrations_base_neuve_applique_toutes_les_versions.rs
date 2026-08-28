//! Cas de test isolé.

use super::*;

#[test]
fn test_migrations_base_neuve_applique_toutes_les_versions() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, DERNIERE_VERSION);
    for table in ["llm_calls", "ats_scores", "ai_cache", "app_kv"] {
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "table {table} absente");
    }
}
