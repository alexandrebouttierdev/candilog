use super::*;

#[test]
fn test_pagination_filtre_par_type_avant_la_limite() {
    let repo = repo();
    let mut esn = entree("Alpha Services");
    esn.type_ = Some("ESN".into());
    let mut cabinet = entree("Beta Conseil");
    cabinet.type_ = Some("Cabinet".into());
    repo.create(&esn).unwrap();
    repo.create(&cabinet).unwrap();

    let page = repo.list_page(1, 24, "", Some("cabinet")).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].name, "Beta Conseil");
    assert_eq!(repo.list_types().unwrap(), vec!["Cabinet", "ESN"]);
}
