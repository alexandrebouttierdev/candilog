//! Cas de test isolé.

use super::*;

/// Les bornes sont inclusives : une candidature envoyée le jour même du début ou de la fin
/// de période doit apparaître, sans quoi le filtre « 30 derniers jours » perdrait ses deux
/// jours extrêmes.
#[test]
fn test_le_filtre_de_periode_borne_les_deux_extremites() {
    let (repo, entreprise_id) = contexte();
    for date in ["2026-08-01", "2026-08-15", "2026-08-31"] {
        repo.create(&entree(entreprise_id, "Développeur", date))
            .unwrap();
    }

    let page = repo
        .list_page(
            1,
            10,
            &FiltreCandidatures {
                date_debut: Some("2026-08-01".into()),
                date_fin: Some("2026-08-15".into()),
                ..FiltreCandidatures::default()
            },
        )
        .unwrap();

    assert_eq!(page.total, 2);
}
