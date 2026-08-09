//! Cas de test isolé.

use super::*;

#[test]
fn test_migration_metier_cree_toutes_les_tables() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    for table in [
        "entreprises",
        "contacts",
        "candidatures",
        "statut_history",
        "relances",
        "entretiens",
        "cv_versions",
        "parametres",
        "profil",
    ] {
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
