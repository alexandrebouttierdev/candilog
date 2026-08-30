//! Cas de test isolé.

use super::*;

#[test]
fn test_migrations_executees_deux_fois_restent_idempotentes() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, LATEST_SCHEMA_VERSION);
}
