use super::*;

#[test]
fn test_pagination_applique_la_recherche_avant_la_limite() {
    let repo = repo();
    for index in 0..30 {
        repo.create(&entree(&format!("Entreprise {index:02}")))
            .unwrap();
    }
    let page = repo.list_page(2, 10, &CompanyFilter::default()).unwrap();
    assert_eq!(page.items.len(), 10);
    assert_eq!(page.total_pages, 3);
    let filtered = repo.list_page(1, 10, &recherche("Entreprise 29")).unwrap();
    assert_eq!(filtered.total, 1);
    assert_eq!(filtered.items[0].name, "Entreprise 29");
}

/// La recherche libre couvre le libellé du secteur, résolu par jointure : sans cela, taper
/// « Informatique » ne trouverait rien alors que la colonne l'affiche.
#[test]
fn la_recherche_libre_atteint_le_libelle_du_secteur() {
    let repo = repo();
    let mut informatique = entree("Alpha Services");
    informatique.sector_id = Some(uuid::Uuid::parse_str(SECTEUR_INFORMATIQUE).unwrap());
    repo.create(&informatique).unwrap();
    repo.create(&entree("Beta Conseil")).unwrap();

    let page = repo.list_page(1, 10, &recherche("informatique")).unwrap();

    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].name, "Alpha Services");
}
