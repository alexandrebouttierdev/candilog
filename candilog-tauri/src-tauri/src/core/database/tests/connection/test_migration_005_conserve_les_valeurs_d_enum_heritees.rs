//! Cas de test isolé.

use super::*;

/// Une base héritée peut porter des valeurs d'énumération absentes des nouveaux `CHECK`.
/// La migration doit les **normaliser**, jamais faire disparaître la ligne : un
/// `INSERT OR IGNORE` convertirait une valeur inattendue en perte silencieuse de données.
#[test]
fn test_migration_005_conserve_les_valeurs_d_enum_heritees() {
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
             INSERT INTO candidatures
                (id, entreprise_id, poste, type_contrat, statut, date_envoi, created_at, updated_at)
                VALUES ('c2', 'e1', 'Ops', 'Portage', 'ACCEPTEE', '2026-01-02', '2026-01-01', '2026-01-01');
             INSERT INTO entretiens
                (id, candidature_id, date_entretien, type, created_at, updated_at)
                VALUES ('i1', 'c2', '2026-01-20', 'Déjeuner', '2026-01-01', '2026-01-01');",
        )
        .unwrap();
    }

    run_local_migrations(&pool).unwrap();

    let conn = pool.get().unwrap();
    assert_eq!(
        compter(&conn, "candidatures"),
        2,
        "une candidature au statut hérité a été supprimée au lieu d'être normalisée"
    );
    assert_eq!(
        compter(&conn, "entretiens"),
        1,
        "un entretien au type hérité a été supprimé au lieu d'être normalisé"
    );

    let (type_contrat, statut): (String, String) = conn
        .query_row(
            "SELECT type_contrat, statut FROM candidatures WHERE id = 'c2'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(type_contrat, "Autre");
    assert_eq!(statut, "EN_ATTENTE");

    let type_entretien: String = conn
        .query_row("SELECT type FROM entretiens WHERE id = 'i1'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(type_entretien, "Autre");
}
