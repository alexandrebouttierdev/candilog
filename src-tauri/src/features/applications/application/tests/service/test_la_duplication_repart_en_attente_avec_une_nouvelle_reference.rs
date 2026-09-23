//! Duplication d'une candidature (`⌘D`).

use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use crate::features::applications::infrastructure::SqliteApplicationRepository;

#[test]
fn la_copie_repart_en_attente_aujourd_hui_sans_emporter_l_historique() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let company_id = uuid::Uuid::new_v4();
    pool.get()
        .unwrap()
        .execute(
            "INSERT INTO companies (id, name, created_at, updated_at)
             VALUES (?1, 'Novéa', '2026-01-01', '2026-01-01')",
            [company_id.to_string()],
        )
        .unwrap();
    let service = ApplicationService::new(SqliteApplicationRepository::new(pool));
    let mut saisie = new("Chargé d'exploitation");
    saisie.company_id = company_id;
    saisie.channel = ApplicationChannel::Network;
    saisie.status = ApplicationStatus::Interview;
    let source = service.create(&saisie).unwrap();

    let copie = service.duplicate(source.id).unwrap();

    assert_ne!(copie.id, source.id);
    assert_eq!(copie.reference_number, source.reference_number + 1);
    assert_eq!(copie.job_title, source.job_title);
    assert_eq!(copie.channel, ApplicationChannel::Network);
    assert_eq!(copie.status, ApplicationStatus::Pending);
    assert_eq!(
        copie.sent_date,
        chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string()
    );
    assert_eq!(service.status_history(copie.id).unwrap().len(), 1);
}
