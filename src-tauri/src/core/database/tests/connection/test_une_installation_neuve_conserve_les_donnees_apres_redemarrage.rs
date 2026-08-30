//! Scénario complet d'une installation neuve, sur un vrai fichier de base.
//!
//! Les autres cas travaillent en mémoire, ce qui ne dit rien de la persistance : ici la base
//! est fermée puis rouverte, comme le fait un redémarrage de Candilog.

use super::*;
use crate::features::applications::domain::{
    ApplicationRepository, ApplicationStatus, ApplicationType, NewApplication, WeeklyWorkSchedule,
};
use crate::features::applications::infrastructure::SqliteApplicationRepository;
use crate::features::companies::domain::{CompanyRepository, CompanySize, NewCompany};
use crate::features::companies::infrastructure::SqliteCompanyRepository;
use crate::features::referentials::domain::ReferentialRepository;
use crate::features::referentials::infrastructure::SqliteReferentialRepository;

#[test]
fn une_installation_neuve_conserve_les_donnees_apres_redemarrage() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("candilog.sqlite");

    // ── Premier démarrage : le fichier n'existe pas encore ──────────────
    assert!(!path.exists());
    validate_database_file(&path).unwrap();
    let pool = open_pool(Some(&path)).unwrap();
    run_local_migrations(&pool).unwrap();

    let referentials = SqliteReferentialRepository::new(pool.clone())
        .load()
        .unwrap();
    assert_eq!(referentials.sectors.len(), 23);
    assert_eq!(referentials.professional_domains.len(), 22);
    assert_eq!(referentials.contract_types.len(), 22);
    assert_eq!(referentials.company_types.len(), 38);

    let companies = SqliteCompanyRepository::new(pool.clone());
    let company = companies
        .create(&NewCompany {
            name: "Nova Digital".into(),
            sector_id: Some(uuid::Uuid::parse_str("5ec70000-0000-4000-8000-000000000003").unwrap()),
            company_type_id: Some("IT_SERVICES_COMPANY".into()),
            company_size: CompanySize::Pme,
            website: None,
            city: Some("Rennes".into()),
            address: Some("12 rue des Lilas".into()),
            notes: None,
        })
        .unwrap();

    let applications = SqliteApplicationRepository::new(pool.clone());
    let application = applications
        .create(&NewApplication {
            job_title: "Développeur Rust".into(),
            company_id: company.id,
            contact_id: None,
            application_type: ApplicationType::JobOffer,
            contract_type_code: "MIS".into(),
            weekly_work_schedule: WeeklyWorkSchedule::PartTime,
            weekly_hours: Some(24.5),
            professional_domain_id: Some("M18".into()),
            city: None,
            address: None,
            company_type_id: Some("FINAL_CLIENT".into()),
            status: ApplicationStatus::Pending,
            sent_date: "2026-08-20".into(),
            job_url: Some("https://example.org/offre".into()),
            notes: None,
        })
        .unwrap();

    // ── Fermeture : le pool est détruit, le fichier reste ───────────────
    drop(applications);
    drop(companies);
    drop(pool);
    assert!(path.exists());

    // ── Second démarrage sur le fichier existant ────────────────────────
    validate_database_file(&path).unwrap();
    let pool = open_pool(Some(&path)).unwrap();
    run_local_migrations(&pool).unwrap();

    let relue = SqliteApplicationRepository::new(pool.clone())
        .get(application.id)
        .unwrap();

    assert_eq!(relue.job_title, "Développeur Rust");
    assert_eq!(relue.contract_type_name.as_deref(), Some("Intérim"));
    assert_eq!(
        relue.professional_domain_name.as_deref(),
        Some("Informatique / Télécommunication")
    );
    assert_eq!(relue.weekly_work_schedule, WeeklyWorkSchedule::PartTime);
    assert_eq!(relue.weekly_hours, Some(24.5));
    assert_eq!(relue.company_size, CompanySize::Pme);
    // Ville héritée de l'entreprise, type d'entreprise surchargé : les deux régimes
    // survivent au redémarrage.
    assert_eq!(relue.city, None);
    assert_eq!(relue.effective_city.as_deref(), Some("Rennes"));
    assert_eq!(relue.effective_address.as_deref(), Some("12 rue des Lilas"));
    assert_eq!(relue.company_type_id.as_deref(), Some("FINAL_CLIENT"));
    assert_eq!(
        relue.effective_company_type_name.as_deref(),
        Some("Client final")
    );

    let entreprise = SqliteCompanyRepository::new(pool.clone())
        .get(company.id)
        .unwrap();
    assert_eq!(
        entreprise.sector_name.as_deref(),
        Some("Banque / Assurance")
    );
    assert_eq!(
        entreprise.company_type_name.as_deref(),
        Some("ESN / Société de services numériques")
    );

    // Les référentiels ne sont ni redoublés ni vidés par le second démarrage.
    let referentials = SqliteReferentialRepository::new(pool).load().unwrap();
    assert_eq!(referentials.contract_types.len(), 22);
    assert_eq!(referentials.sectors.len(), 23);
}
