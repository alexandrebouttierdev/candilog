use super::*;

#[test]
fn test_pagination_filtre_par_type_avant_la_limite() {
    let repo = repo();
    let mut esn = entree("Alpha Services");
    esn.company_type_id = Some("IT_SERVICES_COMPANY".into());
    let mut cabinet = entree("Beta Conseil");
    cabinet.company_type_id = Some("CONSULTING_FIRM".into());
    repo.create(&esn).unwrap();
    repo.create(&cabinet).unwrap();

    let page = repo
        .list_page(
            1,
            24,
            &CompanyFilter {
                company_type_id: Some("CONSULTING_FIRM".into()),
                ..CompanyFilter::default()
            },
        )
        .unwrap();

    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].name, "Beta Conseil");
    assert_eq!(
        page.items[0].company_type_name.as_deref(),
        Some("Cabinet de conseil")
    );
}

#[test]
fn le_filtre_par_secteur_precede_la_limite() {
    let repo = repo();
    let mut informatique = entree("Alpha Services");
    informatique.sector_id = Some(uuid::Uuid::parse_str(SECTEUR_INFORMATIQUE).unwrap());
    repo.create(&informatique).unwrap();
    repo.create(&entree("Beta Conseil")).unwrap();

    let page = repo
        .list_page(
            1,
            24,
            &CompanyFilter {
                sector_id: Some(uuid::Uuid::parse_str(SECTEUR_INFORMATIQUE).unwrap()),
                ..CompanyFilter::default()
            },
        )
        .unwrap();

    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].name, "Alpha Services");
}
