//! Une base vide obtient l'intégralité du modèle en une seule application du schéma.

use super::*;

#[test]
fn test_le_schema_cree_toutes_les_tables() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    for table in [
        "companies",
        "contacts",
        "applications",
        "status_history",
        "follow_ups",
        "interviews",
        "resume_versions",
        "cover_letters",
        "sectors",
        "professional_domains",
        "company_types",
        "contract_types",
        "settings",
        "profile",
        "llm_calls",
        "ats_scores",
        "app_kv",
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

    // `ai_cache` a été retirée : rien ne l'alimentait, et l'écran des réglages proposait de
    // la vider en annonçant un effet qui ne se produisait jamais (docs/AI.md).
    let cache: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='ai_cache'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(cache, 0, "ai_cache aurait dû disparaître du schéma");
}

/// Les colonnes du nouveau modèle sont présentes, et les anciennes ont bien disparu : une
/// colonne obsolète laissée en place redeviendrait une seconde source de vérité.
#[test]
fn les_colonnes_obsoletes_ont_disparu() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();

    let colonnes = |table: &str| -> Vec<String> {
        let mut query = conn
            .prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
            .unwrap();
        let rows = query.query_map([], |row| row.get(0)).unwrap();
        rows.map(Result::unwrap).collect()
    };

    let companies = colonnes("companies");
    for attendue in [
        "sector_id",
        "company_type_id",
        "company_size",
        "city",
        "address",
    ] {
        assert!(
            companies.contains(&attendue.to_owned()),
            "companies.{attendue} absente"
        );
    }
    for obsolete in ["sector", "type"] {
        assert!(
            !companies.contains(&obsolete.to_owned()),
            "companies.{obsolete} aurait dû disparaître"
        );
    }

    let applications = colonnes("applications");
    for attendue in [
        "application_type",
        "contract_type_code",
        "weekly_work_schedule",
        "weekly_hours",
        "professional_domain_id",
        "city",
        "address",
        "company_type_id",
    ] {
        assert!(
            applications.contains(&attendue.to_owned()),
            "applications.{attendue} absente"
        );
    }
    for obsolete in ["contract_type", "contract_duration_months", "company_type"] {
        assert!(
            !applications.contains(&obsolete.to_owned()),
            "applications.{obsolete} aurait dû disparaître"
        );
    }
}
