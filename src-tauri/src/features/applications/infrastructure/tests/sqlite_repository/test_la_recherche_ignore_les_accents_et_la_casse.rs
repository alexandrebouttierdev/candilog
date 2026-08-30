//! Cas de test isolé.

use super::*;

/// `lower()` de SQLite n'agit que sur l'ASCII : il laisse « É » intact. Une recherche
/// normalisée côté Rust ne rencontrait donc jamais la majuscule accentuée stockée, et une
/// entreprise nommée « ÉCOLE DIRECTE » restait introuvable — y compris en la cherchant par
/// son propre nom.
#[test]
fn test_la_recherche_ignore_les_accents_et_la_casse() {
    let (repo, _) = context();
    let ecole = autre_entreprise(&repo, "ÉCOLE DIRECTE", "Rennes", "IT_SERVICES_COMPANY");
    repo.create(&entree(ecole, "Développeur ÉLECTRONIQUE", "2026-08-20"))
        .unwrap();

    for terme in ["école", "ECOLE", "Écolé", "électronique", "ELECTRONIQUE"] {
        let page = repo
            .list_page(
                1,
                10,
                &ApplicationFilter {
                    search: terme.into(),
                    ..ApplicationFilter::default()
                },
            )
            .unwrap();
        assert_eq!(page.total, 1, "recherche « {terme} » sans résultat");
    }
}
