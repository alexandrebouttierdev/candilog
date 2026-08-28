//! Cas de test isolé.

use super::*;

#[test]
fn la_migration_008_cree_la_table_et_la_colonne_de_liaison() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();

    let table: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'sectors'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(table, 1, "table sectors absente");

    let column: i64 = conn
        .query_row(
            "SELECT count(*) FROM pragma_table_info('companies') WHERE name = 'sector_id'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(column, 1, "colonne companies.sector_id absente");

    let index: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_companies_sector_id'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(index, 1, "index idx_companies_sector_id absent");
}
