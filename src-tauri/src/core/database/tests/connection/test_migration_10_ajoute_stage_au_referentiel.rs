//! Cas de test isolé.

use super::*;

#[test]
fn migration_10_ajoute_stage_a_une_base_existante() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    {
        let conn = pool.get().unwrap();
        conn.execute("DELETE FROM contract_types WHERE code = 'STAGE'", [])
            .unwrap();
        conn.pragma_update(None, "user_version", 9).unwrap();
    }

    run_local_migrations(&pool).unwrap();

    let conn = pool.get().unwrap();
    let stage: (String, i64) = conn
        .query_row(
            "SELECT name, sort_order FROM contract_types WHERE code = 'STAGE'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(stage, ("Stage".into(), 23));
}
