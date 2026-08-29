//! Cas de test isolé.

use super::*;

/// Cocher deux statuts doit les garder tous les deux, pas remplacer le premier.
#[test]
fn test_le_filtre_par_statuts_retient_toutes_les_valeurs_cochees() {
    let (repo, company_id) = context();
    for (titre, status) in [
        ("Alpha", ApplicationStatus::Pending),
        ("Beta", ApplicationStatus::Interview),
        ("Gamma", ApplicationStatus::Rejected),
    ] {
        let mut input = entree(company_id, titre, "2026-08-20");
        input.status = status;
        repo.create(&input).unwrap();
    }

    let page = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                status: vec![ApplicationStatus::Interview, ApplicationStatus::Rejected],
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
    assert!(titres.contains(&"Beta"));
    assert!(titres.contains(&"Gamma"));
    assert!(!titres.contains(&"Alpha"));
}
