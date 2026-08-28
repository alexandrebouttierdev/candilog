//! Cas de test isolé.

use super::*;

/// Migre une base `user_version = 4` peuplée de données métier reliées entre elles et
/// exige l'égalité stricte des comptages avant et après.
///
/// La migration 005 recrée `candidatures` par DROP/RENAME ; `relances`, `entretiens` et
/// `statut_history` la référencent en `ON DELETE CASCADE`. Avec `PRAGMA foreign_keys = ON`
/// — que `initialiser_connexion` pose sur chaque connexion du pool — le DROP déclenche un
/// DELETE implicite, donc les cascades : les trois tables enfants sont vidées.
#[test]
fn test_migration_005_preserve_les_donnees_metier_liees() {
    let pool = open_pool(None).unwrap();
    {
        let conn = pool.get().unwrap();
        preparer_base_version_quatre(&conn);
        conn.execute_batch(
            "INSERT INTO entreprises (id, nom, created_at, updated_at)
                VALUES ('e1', 'Acme', '2026-01-01', '2026-01-01');
             INSERT INTO candidatures
                (id, entreprise_id, poste, type_contrat, statut, date_envoi, created_at, updated_at)
                VALUES ('c1', 'e1', 'Dev', 'CDI', 'EN_ATTENTE', '2026-01-01', '2026-01-01', '2026-01-01');
             INSERT INTO relances (id, candidature_id, date_relance, type, created_at)
                VALUES ('r1', 'c1', '2026-01-08', 'Email', '2026-01-01');
             INSERT INTO relances (id, candidature_id, date_relance, type, created_at)
                VALUES ('r2', 'c1', '2026-01-15', 'Email', '2026-01-01');
             INSERT INTO entretiens
                (id, candidature_id, date_entretien, type, created_at, updated_at)
                VALUES ('i1', 'c1', '2026-01-20', 'Visio', '2026-01-01', '2026-01-01');
             INSERT INTO statut_history (id, candidature_id, statut, changed_at)
                VALUES ('h1', 'c1', 'EN_ATTENTE', '2026-01-01');",
        )
        .unwrap();
    }

    run_local_migrations(&pool).unwrap();

    let conn = pool.get().unwrap();
    for (table, attendu) in [
        ("candidatures", 1_i64),
        ("relances", 2),
        ("entretiens", 1),
        ("statut_history", 1),
    ] {
        assert_eq!(
            compter(&conn, table),
            attendu,
            "la migration a effacé des lignes de {table}"
        );
    }
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, DERNIERE_VERSION);
}
