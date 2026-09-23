//! Cas de test isolé.

use super::*;

/// Base au schéma v2 contenant trois candidatures, dont une spontanée.
fn base_v2() -> SqlitePool {
    let pool = open_pool(None).unwrap();
    {
        let conn = pool.get().unwrap();
        conn.execute_batch(MIGRATIONS[0].1).unwrap();
        conn.execute_batch(MIGRATIONS[1].1).unwrap();
        conn.pragma_update(None, "user_version", 2).unwrap();
        conn.execute_batch(
            "INSERT INTO companies (id, name, created_at, updated_at)
             VALUES ('e-1', 'Novéa', '2026-01-01', '2026-01-01');
             INSERT INTO applications (id, company_id, job_title, application_type,
                contract_type_code, status, sent_date, job_url, created_at, updated_at)
             VALUES
                ('c-tard', 'e-1', 'Troisième', 'OFFRE', 'CDI', 'EN_ATTENTE', '2026-03-01',
                 'https://exemple.fr/3', '2026-03-01T10:00:00', '2026-03-01T10:00:00'),
                ('c-tot', 'e-1', 'Première', 'OFFRE', 'CDI', 'EN_ATTENTE', '2026-01-01',
                 'https://exemple.fr/1', '2026-01-01T10:00:00', '2026-01-01T10:00:00'),
                ('c-milieu', 'e-1', 'Deuxième', 'SPONTANEE', 'CDI', 'EN_ATTENTE', '2026-02-01',
                 NULL, '2026-02-01T10:00:00', '2026-02-01T10:00:00');",
        )
        .unwrap();
    }
    pool
}

fn reference(conn: &rusqlite::Connection, id: &str) -> i64 {
    conn.query_row(
        "SELECT reference_number FROM applications WHERE id = ?1",
        [id],
        |row| row.get(0),
    )
    .unwrap()
}

fn canal(conn: &rusqlite::Connection, id: &str) -> String {
    conn.query_row(
        "SELECT channel FROM applications WHERE id = ?1",
        [id],
        |row| row.get(0),
    )
    .unwrap()
}

#[test]
fn les_candidatures_existantes_sont_numerotees_par_ordre_de_creation() {
    let pool = base_v2();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    assert_eq!(reference(&conn, "c-tot"), 1);
    assert_eq!(reference(&conn, "c-milieu"), 2);
    assert_eq!(reference(&conn, "c-tard"), 3);
}

#[test]
fn le_canal_reprend_la_nature_de_la_candidature() {
    let pool = base_v2();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    assert_eq!(canal(&conn, "c-tot"), "OFFER");
    assert_eq!(canal(&conn, "c-milieu"), "SPONTANEOUS");
}

#[test]
fn une_nouvelle_candidature_recoit_le_numero_suivant_meme_apres_une_suppression() {
    let pool = base_v2();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    // Un numéro supprimé n'est jamais réattribué : « CAN-003 » désignerait sinon deux
    // candidatures différentes dans l'historique de l'utilisateur.
    conn.execute("DELETE FROM applications WHERE id = 'c-tard'", [])
        .unwrap();
    conn.execute_batch(
        "INSERT INTO applications (id, company_id, job_title, contract_type_code, sent_date,
            job_url, created_at, updated_at)
         VALUES ('c-neuve', 'e-1', 'Quatrième', 'CDI', '2026-04-01', 'https://exemple.fr/4',
            '2026-04-01T10:00:00', '2026-04-01T10:00:00');",
    )
    .unwrap();
    assert_eq!(reference(&conn, "c-neuve"), 3);
}

#[test]
fn deux_candidatures_ne_partagent_jamais_une_reference() {
    let pool = base_v2();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    let doublon = conn.execute(
        "UPDATE applications SET reference_number = 1 WHERE id = 'c-milieu'",
        [],
    );
    assert!(doublon.is_err(), "l'index unique doit refuser un doublon");
}

#[test]
fn un_canal_hors_catalogue_est_refuse() {
    let pool = base_v2();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    let refus = conn.execute(
        "UPDATE applications SET channel = 'JOB_BOARD' WHERE id = 'c-tot'",
        [],
    );
    assert!(refus.is_err());
}
