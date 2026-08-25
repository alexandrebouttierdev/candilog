//! Cas de test isolé.

use super::*;

#[test]
fn test_open_pool_memoire_partage_les_donnees_entre_connexions() {
    // Régression : `SqliteConnectionManager::memory()` donne une base isolée
    // par connexion, ce qui casse tout repository lisant après écriture.
    let pool = open_pool(None).unwrap();
    let a = pool.get().unwrap();
    a.execute_batch("CREATE TABLE marqueur (x INTEGER);")
        .unwrap();
    a.execute("INSERT INTO marqueur (x) VALUES (42)", [])
        .unwrap();
    drop(a);
    let b = pool.get().unwrap();
    let x: i64 = b
        .query_row("SELECT x FROM marqueur", [], |r| r.get(0))
        .unwrap();
    assert_eq!(x, 42);
}
