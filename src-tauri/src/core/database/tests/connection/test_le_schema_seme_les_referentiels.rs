//! Une base neuve démarre avec ses quatre référentiels déjà peuplés.

use super::*;

#[test]
fn les_quatre_referentiels_sont_peuples_des_le_premier_demarrage() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();

    for (table, attendu) in [
        ("sectors", 23),
        ("professional_domains", 22),
        ("company_types", 38),
        ("contract_types", 22),
    ] {
        let total: i64 = conn
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(total, attendu, "{table} mal semé");
    }
}

/// Un second démarrage rejoue les mêmes semences : rien ne doit être dupliqué, et les
/// migrations restent silencieuses puisque le curseur est déjà à jour.
#[test]
fn un_second_demarrage_ne_duplique_aucune_semence() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    {
        let conn = pool.get().unwrap();
        conn.execute_batch(include_str!("../../../../../migrations/init_schema.sql"))
            .unwrap();
    }
    run_local_migrations(&pool).unwrap();

    let conn = pool.get().unwrap();
    let total: i64 = conn
        .query_row("SELECT count(*) FROM contract_types", [], |row| row.get(0))
        .unwrap();
    assert_eq!(total, 22);
}
