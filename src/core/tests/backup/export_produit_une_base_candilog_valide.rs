//! Cas de test isolé.

use super::*;

#[test]
fn export_produit_une_base_candilog_valide() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("backup.sqlite");
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    export(&pool, &path).unwrap();
    validate(&path).unwrap();
}
