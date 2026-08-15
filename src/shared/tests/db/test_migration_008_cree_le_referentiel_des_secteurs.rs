//! Cas de test isolé.

use super::*;

#[test]
fn la_migration_008_cree_la_table_et_la_colonne_de_liaison() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();

    let table: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'secteurs_activite'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(table, 1, "table secteurs_activite absente");

    let colonne: i64 = conn
        .query_row(
            "SELECT count(*) FROM pragma_table_info('entreprises') WHERE name = 'secteur_id'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(colonne, 1, "colonne entreprises.secteur_id absente");

    let index: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_entreprises_secteur_id'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(index, 1, "index idx_entreprises_secteur_id absent");
}
