use super::*;

#[test]
fn test_pagination_accede_aux_elements_apres_deux_cents() {
    let repo = repo();
    for index in 0..205 {
        repo.create(&entree(&format!("Entreprise {index:03}")))
            .unwrap();
    }
    let page = repo.list_page(6, 40, &CompanyFilter::default()).unwrap();
    assert_eq!(page.total, 205);
    assert_eq!(page.total_pages, 6);
    assert_eq!(page.items.len(), 5);
    assert_eq!(page.items[0].name, "Entreprise 200");
}
