//! Cas de test isolé.

use super::*;

/// L'export d'une sélection ne doit pas réécrire tout le filtre courant.
#[test]
fn test_le_filtre_par_identifiants_restreint_l_export() {
    let (repo, company_id) = context();
    let premiere = repo
        .create(&entree(company_id, "Alpha", "2026-08-01"))
        .unwrap();
    let _deuxieme = repo
        .create(&entree(company_id, "Beta", "2026-08-02"))
        .unwrap();
    let troisieme = repo
        .create(&entree(company_id, "Gamma", "2026-08-03"))
        .unwrap();

    let page = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                ids: vec![premiere.id, troisieme.id],
                ..ApplicationFilter::default()
            },
        )
        .unwrap();

    assert_eq!(page.total, 2);
    let titres: Vec<_> = page
        .items
        .iter()
        .map(|item| item.job_title.as_str())
        .collect();
    assert!(titres.contains(&"Alpha"));
    assert!(titres.contains(&"Gamma"));
    assert!(!titres.contains(&"Beta"));
}
