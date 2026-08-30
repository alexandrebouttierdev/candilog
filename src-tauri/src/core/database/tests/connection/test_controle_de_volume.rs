use super::*;
use crate::core::database::helpers::connection;
use crate::features::applications::domain::{ApplicationFilter, ApplicationRepository};
use crate::features::applications::infrastructure::SqliteApplicationRepository;
use crate::features::companies::domain::{CompanyFilter, CompanyRepository};
use crate::features::companies::infrastructure::SqliteCompanyRepository;
use crate::features::contacts::domain::ContactRepository;
use crate::features::contacts::infrastructure::SqliteContactRepository;
use crate::features::documents::domain::ResumeRepository;
use crate::features::documents::infrastructure::SqliteResumeRepository;

#[test]
#[ignore = "contrôle de volume explicite"]
fn controle_de_volume_pagination_et_indexes() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let mut conn = connection(&pool).unwrap();
    let tx = conn.transaction().unwrap();
    let company_anchor = uuid::Uuid::nil().to_string();

    {
        let mut insert = tx
            .prepare(
                "INSERT INTO companies (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            )
            .unwrap();
        insert
            .execute(rusqlite::params![
                company_anchor,
                "Entreprise-00000",
                "2026-08-30T00:00:00Z"
            ])
            .unwrap();
        for index in 1_u128..10_000 {
            insert
                .execute(rusqlite::params![
                    uuid::Uuid::from_u128(index).to_string(),
                    format!("Entreprise-{index:05}"),
                    "2026-08-30T00:00:00Z",
                ])
                .unwrap();
        }
    }
    {
        let mut insert = tx
            .prepare(
                "INSERT INTO applications (id, company_id, job_title, contract_type_code, status,
                sent_date, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'CDI', 'EN_ATTENTE', '2026-08-30', ?4, ?4)",
            )
            .unwrap();
        for index in 0_u128..10_000 {
            insert
                .execute(rusqlite::params![
                    uuid::Uuid::from_u128(20_000 + index).to_string(),
                    company_anchor,
                    format!("Poste-{index:05}"),
                    "2026-08-30T00:00:00Z",
                ])
                .unwrap();
        }
    }
    {
        let mut insert = tx.prepare(
            "INSERT INTO contacts (id, first_name, name, created_at, updated_at) VALUES (?1, 'Camille', ?2, ?3, ?3)",
        ).unwrap();
        for index in 0_u128..10_000 {
            insert
                .execute(rusqlite::params![
                    uuid::Uuid::from_u128(40_000 + index).to_string(),
                    format!("Contact-{index:05}"),
                    "2026-08-30T00:00:00Z",
                ])
                .unwrap();
        }
    }
    {
        let mut insert = tx.prepare(
            "INSERT INTO resume_versions (id, name, content, created_at) VALUES (?1, ?2, '{}', ?3)",
        ).unwrap();
        for index in 0_u128..10_000 {
            insert
                .execute(rusqlite::params![
                    uuid::Uuid::from_u128(60_000 + index).to_string(),
                    format!("CV-{index:05}"),
                    format!("2026-08-30T00:{:02}:{:02}Z", (index / 60) % 60, index % 60),
                ])
                .unwrap();
        }
    }
    tx.commit().unwrap();

    let companies = SqliteCompanyRepository::new(pool.clone())
        .list_page(1_250, 8, &CompanyFilter::default())
        .unwrap();
    assert_eq!(companies.total, 10_000);
    assert_eq!(companies.items.len(), 8);
    let contacts = SqliteContactRepository::new(pool.clone())
        .list_page(1_250, 8, "", None)
        .unwrap();
    assert_eq!(contacts.total, 10_000);
    assert_eq!(contacts.items.len(), 8);
    let applications = SqliteApplicationRepository::new(pool.clone())
        .list_page(1_250, 8, &ApplicationFilter::default())
        .unwrap();
    assert_eq!(applications.total, 10_000);
    assert_eq!(applications.items.len(), 8);
    let resumes = SqliteResumeRepository::new(pool.clone())
        .list_page(1, 8, "CV-09999")
        .unwrap();
    assert_eq!(resumes.total, 1);
    assert_eq!(resumes.items[0].name, "CV-09999");

    let conn = connection(&pool).unwrap();
    for (sql, expected_index) in [
        ("EXPLAIN QUERY PLAN SELECT id, name, created_at FROM resume_versions ORDER BY created_at DESC LIMIT 8", "idx_resume_versions_created_at"),
        ("EXPLAIN QUERY PLAN SELECT id, name, created_at FROM cover_letters ORDER BY created_at DESC LIMIT 8", "idx_cover_letters_created_at"),
    ] {
        let mut query = conn.prepare(sql).unwrap();
        let plan = query.query_map([], |row| row.get::<_, String>(3)).unwrap().map(Result::unwrap).collect::<Vec<_>>();
        assert!(plan.iter().any(|line| line.contains(expected_index)), "plan sans {expected_index}: {plan:?}");
    }
}
