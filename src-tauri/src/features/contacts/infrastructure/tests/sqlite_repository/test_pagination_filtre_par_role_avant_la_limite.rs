use super::*;

#[test]
fn test_pagination_filtre_par_role_avant_la_limite() {
    let repo = repo();
    let mut recruteur = entree("Durand", None);
    recruteur.tracking_role = Some("Recruteur".into());
    let mut manager = entree("Martin", None);
    manager.tracking_role = Some("Manager".into());
    repo.create(&recruteur).unwrap();
    repo.create(&manager).unwrap();

    let page = repo.list_page(1, 24, "", Some("recruteur")).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].name, "Durand");
}
