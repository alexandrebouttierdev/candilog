use super::*;

#[test]
fn test_pagination_applique_la_recherche_avant_la_limite() {
    let repo = repo();
    for index in 0..30 {
        repo.create(&entree(&format!("Entreprise {index:02}")))
            .unwrap();
    }
    let page = repo.list_page(2, 10, "").unwrap();
    assert_eq!(page.items.len(), 10);
    assert_eq!(page.total_pages, 3);
    let filtered = repo.list_page(1, 10, "Entreprise 29").unwrap();
    assert_eq!(filtered.total, 1);
    assert_eq!(filtered.items[0].nom, "Entreprise 29");
}
