//! Cas de test isolé.

use super::*;

#[test]
fn test_connexion_du_pool_active_les_cles_etrangeres() {
    let pool = open_pool(None).unwrap();
    let conn = pool.get().unwrap();
    let actives: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(actives, 1);
}
