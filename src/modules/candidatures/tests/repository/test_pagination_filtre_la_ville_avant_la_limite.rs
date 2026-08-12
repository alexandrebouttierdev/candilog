use super::*;

#[test]
fn test_pagination_filtre_la_ville_avant_la_limite() {
    let repo = repo();
    for index in 0..60 {
        let company = entreprise(&repo, &format!("Entreprise {index:02}"));
        let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
        conn.execute(
            "UPDATE entreprises SET ville = ?2 WHERE id = ?1",
            rusqlite::params![
                company.to_string(),
                if index % 2 == 0 { "Rennes" } else { "Lyon" }
            ],
        )
        .unwrap();
        drop(conn);
        repo.create(&entree(company, &format!("Poste {index:02}")))
            .unwrap();
    }

    let query = CandidaturePageQuery {
        city: "rennes".into(),
        descending: true,
        ..CandidaturePageQuery::default()
    };
    let page = repo.list_page(1, 24, &query).unwrap();
    assert_eq!(page.total, 30);
    assert_eq!(page.items.len(), 24);
    assert!(page.items.iter().all(|item| {
        item.poste
            .strip_prefix("Poste ")
            .and_then(|value| value.parse::<usize>().ok())
            .is_some_and(|index| index % 2 == 0)
    }));
}
