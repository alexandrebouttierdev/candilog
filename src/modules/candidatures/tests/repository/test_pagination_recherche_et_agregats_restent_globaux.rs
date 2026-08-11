use super::*;

#[test]
fn test_pagination_recherche_et_agregats_restent_globaux() {
    let repo = repo();
    let company = entreprise(&repo, "ACME");
    for index in 0..30 {
        repo.create(&entree(company, &format!("Poste {index:02}")))
            .unwrap();
    }

    let page = repo
        .list_page(2, 10, &CandidaturePageQuery::default())
        .unwrap();
    assert_eq!(page.items.len(), 10);
    assert_eq!(page.total, 30);
    assert_eq!(page.total_pages, 3);

    let query = CandidaturePageQuery {
        search: "Poste 29".into(),
        ..CandidaturePageQuery::default()
    };
    let filtered = repo.list_page(1, 10, &query).unwrap();
    assert_eq!(filtered.total, 1);
    assert_eq!(filtered.items[0].poste, "Poste 29");
    assert_eq!(repo.stats().unwrap().total, 30);
}
