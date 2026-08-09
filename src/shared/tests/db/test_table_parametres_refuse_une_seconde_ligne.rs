//! Cas de test isolé.

use super::*;

#[test]
fn test_table_parametres_refuse_une_seconde_ligne() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    conn.execute(
        "INSERT INTO parametres (id, data, updated_at) VALUES (1, '{}', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    let erreur = conn.execute(
        "INSERT INTO parametres (id, data, updated_at) VALUES (2, '{}', '2026-01-01T00:00:00Z')",
        [],
    );
    assert!(erreur.is_err(), "CHECK (id = 1) non appliqué");
}
