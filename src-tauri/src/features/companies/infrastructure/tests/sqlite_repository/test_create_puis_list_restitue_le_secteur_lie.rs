//! Cas de test isolé.

use super::*;
use crate::features::sectors::domain::SectorRepository;
use crate::features::sectors::infrastructure::SqliteSectorRepository;

#[test]
fn create_puis_list_restitue_le_secteur_lie() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let sectors = SqliteSectorRepository::new(pool.clone());
    sectors.ensure_catalog().unwrap();
    let repo = SqliteCompanyRepository::new(pool);
    let reference = sectors.list().unwrap().remove(0);

    let creee = repo
        .create(&NewCompany {
            name: "Agrial".into(),
            sector_id: Some(reference.id),
            sector: Some(reference.name.clone()),
            type_: None,
            website: None,
            city: None,
            address: None,
            notes: None,
        })
        .unwrap();
    assert_eq!(creee.sector_id, Some(reference.id));
    assert_eq!(creee.sector.as_deref(), Some(reference.name.as_str()));

    let list = repo.list().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].sector_id, Some(reference.id));
    assert_eq!(list[0].sector.as_deref(), Some(reference.name.as_str()));
}
