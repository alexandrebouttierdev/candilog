use super::*;

#[test]
fn test_migration_006_indexe_les_dates() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    for index in [
        "idx_applications_date",
        "idx_follow_ups_date",
        "idx_interviews_date",
    ] {
        let found: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
                [index],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(found, 1, "index {index} absent");
    }
}
