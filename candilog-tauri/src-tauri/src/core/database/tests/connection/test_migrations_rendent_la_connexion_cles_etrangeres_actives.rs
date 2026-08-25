//! Cas de test isolé.

use super::*;

/// `run_local_migrations` désactive `foreign_keys` le temps de recréer les tables. Le réglage
/// est **propre à la connexion**, et celle-ci retourne au pool : s'il n'était pas restauré,
/// l'application appliquerait ensuite ses contraintes de façon aléatoire selon la connexion
/// tirée du pool — les suppressions ne cascaderaient plus et les références seraient acceptées.
#[test]
fn test_migrations_rendent_la_connexion_cles_etrangeres_actives() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();

    // Le pool ayant `min_idle(1)`, la connexion rendue est celle qui a migré.
    let conn = pool.get().unwrap();
    let actif: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        actif, 1,
        "la migration a rendu au pool une connexion sans clés étrangères"
    );
}
