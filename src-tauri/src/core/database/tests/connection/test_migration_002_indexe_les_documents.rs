//! Une base Candilog de la génération courante v1 migre vers les index documentaires v2.

use super::*;

#[test]
fn base_courante_v1_migre_vers_v2() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    {
        let conn = pool.get().unwrap();
        conn.execute_batch(
            "DROP INDEX IF EXISTS idx_resume_versions_created_at;
             DROP INDEX IF EXISTS idx_cover_letters_created_at;
             PRAGMA user_version = 1;",
        )
        .unwrap();
    }

    run_local_migrations(&pool).unwrap();

    let conn = pool.get().unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    let indexes: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='index' AND name IN (
                'idx_resume_versions_created_at', 'idx_cover_letters_created_at'
             )",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, 2);
    assert_eq!(indexes, 2);
}
