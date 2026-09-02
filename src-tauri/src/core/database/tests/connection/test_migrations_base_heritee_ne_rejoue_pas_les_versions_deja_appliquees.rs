//! Cas de test isolé.

use super::*;

#[test]
fn test_migrations_base_heritee_ne_rejoue_pas_les_versions_deja_appliquees() {
    let pool = open_pool(None).unwrap();
    {
        let conn = pool.get().unwrap();
        // Base déjà au schéma courant : `init_schema` ne doit pas être rejoué.
        conn.execute_batch(
            "CREATE TABLE app_kv (kv_key TEXT PRIMARY KEY, kv_value TEXT NOT NULL);
                 INSERT INTO app_kv (kv_key, kv_value) VALUES ('temoin', 'intact');",
        )
        .unwrap();
        conn.pragma_update(None, "user_version", LATEST_SCHEMA_VERSION)
            .unwrap();
    }
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    let temoin: String = conn
        .query_row(
            "SELECT kv_value FROM app_kv WHERE kv_key = 'temoin'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        temoin, "intact",
        "une migration déjà appliquée a été rejouée"
    );
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, LATEST_SCHEMA_VERSION);
}
