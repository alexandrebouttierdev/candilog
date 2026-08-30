//! Les index servant les filtres, les jointures et les tris sont créés par le schéma.

use super::*;

#[test]
fn test_le_schema_cree_les_index_utiles() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();

    for index in [
        "idx_applications_company",
        "idx_applications_contact",
        "idx_applications_status",
        "idx_applications_date",
        "idx_applications_type",
        "idx_applications_contract",
        "idx_applications_domain",
        "idx_applications_company_type",
        "idx_applications_schedule",
        "idx_companies_sector_id",
        "idx_companies_company_type",
        "idx_companies_size",
        "idx_contacts_company",
        "idx_status_history_application",
        "idx_follow_ups_application",
        "idx_follow_ups_date",
        "idx_interviews_application",
        "idx_interviews_date",
        "idx_resume_versions_created_at",
        "idx_cover_letters_created_at",
    ] {
        let found: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
                [index],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(found, 1, "index {index} absent");
    }
}
