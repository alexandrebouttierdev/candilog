//! Le schéma protège lui-même ses invariants : le service n'est pas la seule barrière.

use super::*;

/// Base migrée portant une entreprise et une candidature valides.
fn base() -> SqlitePool {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    pool.get()
        .unwrap()
        .execute_batch(
            "INSERT INTO companies (id, name, city, created_at, updated_at)
                VALUES ('e1', 'ACME', 'Rennes', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
             INSERT INTO applications (id, company_id, job_title, contract_type_code, sent_date,
                    created_at, updated_at)
                VALUES ('c1', 'e1', 'Dev', 'CDI', '2026-01-01', '2026-01-01T00:00:00Z',
                    '2026-01-01T00:00:00Z');",
        )
        .unwrap();
    pool
}

#[test]
fn une_taille_d_entreprise_hors_catalogue_est_refusee() {
    let pool = base();
    let conn = pool.get().unwrap();

    assert!(conn
        .execute(
            "UPDATE companies SET company_size = 'GEANTE' WHERE id = 'e1'",
            []
        )
        .is_err());
}

#[test]
fn une_nature_de_candidature_hors_catalogue_est_refusee() {
    let pool = base();
    let conn = pool.get().unwrap();

    assert!(conn
        .execute(
            "UPDATE applications SET application_type = 'COOPTATION' WHERE id = 'c1'",
            []
        )
        .is_err());
}

/// La règle « pas de lien pour une spontanée » vaut aussi en base : le service la fait
/// respecter, mais une écriture qui le contournerait doit être refusée ici.
#[test]
fn une_candidature_spontanee_ne_peut_pas_porter_de_lien_d_offre() {
    let pool = base();
    let conn = pool.get().unwrap();

    assert!(conn
        .execute(
            "UPDATE applications SET application_type = 'SPONTANEE',
                job_url = 'https://example.org/offre' WHERE id = 'c1'",
            []
        )
        .is_err());

    conn.execute(
        "UPDATE applications SET application_type = 'SPONTANEE', job_url = NULL WHERE id = 'c1'",
        [],
    )
    .unwrap();
}

#[test]
fn un_code_de_contrat_hors_referentiel_est_refuse() {
    let pool = base();
    let conn = pool.get().unwrap();

    assert!(conn
        .execute(
            "UPDATE applications SET contract_type_code = 'INEXISTANT' WHERE id = 'c1'",
            []
        )
        .is_err());
}

/// Supprimer une entrée de référentiel ne doit jamais emporter la candidature : le domaine
/// professionnel repasse simplement à « non renseigné ».
#[test]
fn la_suppression_d_un_domaine_professionnel_detache_la_candidature() {
    let pool = base();
    let conn = pool.get().unwrap();
    conn.execute(
        "UPDATE applications SET professional_domain_id = 'M18' WHERE id = 'c1'",
        [],
    )
    .unwrap();

    conn.execute("DELETE FROM professional_domains WHERE code = 'M18'", [])
        .unwrap();

    let domaine: Option<String> = conn
        .query_row(
            "SELECT professional_domain_id FROM applications WHERE id = 'c1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(domaine, None);
}

/// Le ton et la longueur d'une lettre sont un catalogue fermé, interprété au rendu. Sans
/// `CHECK`, seule la couche service les vérifiait, et un `INSERT` direct suffisait à
/// persister une valeur que plus rien ne saurait relire.
#[test]
fn un_ton_ou_une_longueur_de_lettre_hors_catalogue_est_refuse() {
    let pool = base();
    let conn = pool.get().unwrap();
    conn.execute(
        "INSERT INTO cover_letters (id, name, tone, length, content, created_at)
         VALUES ('l1', 'Lettre', 'formal', 'medium', 'Bonjour', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();

    assert!(conn
        .execute(
            "UPDATE cover_letters SET tone = 'sarcastique' WHERE id = 'l1'",
            []
        )
        .is_err());
    assert!(conn
        .execute(
            "UPDATE cover_letters SET length = 'interminable' WHERE id = 'l1'",
            []
        )
        .is_err());
}
