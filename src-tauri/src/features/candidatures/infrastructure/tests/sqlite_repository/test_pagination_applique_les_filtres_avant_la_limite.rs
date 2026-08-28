//! Cas de test isolé.

use super::*;

/// Filtrer après avoir découpé la page rendrait le total faux et laisserait des pages vides
/// au milieu de la liste.
#[test]
fn test_pagination_applique_les_filtres_avant_la_limite() {
    let (repo, entreprise_id) = contexte();
    for index in 0..25 {
        repo.create(&entree(
            entreprise_id,
            &format!("Poste {index:02}"),
            "2026-08-20",
        ))
        .unwrap();
    }

    let page = repo
        .list_page(2, 10, &FiltreCandidatures::default())
        .unwrap();
    assert_eq!(page.items.len(), 10);
    assert_eq!(page.total, 25);
    assert_eq!(page.total_pages, 3);

    let filtree = repo
        .list_page(
            1,
            10,
            &FiltreCandidatures {
                search: "Poste 24".into(),
                ..FiltreCandidatures::default()
            },
        )
        .unwrap();
    assert_eq!(filtree.total, 1);
    assert_eq!(filtree.items[0].poste, "Poste 24");
}
