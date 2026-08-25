//! Cas de test isolé.

use super::*;
use crate::features::candidatures::domain::TriCandidature;

/// Un tri sensible à la casse placerait « atlas » après « Zenith » : l'ordre serait celui
/// des codes ASCII, pas celui que lit l'utilisateur.
#[test]
fn test_le_tri_par_entreprise_ignore_la_casse() {
    let (repo, premiere) = contexte();
    let conn = connexion(&repo.pool).unwrap();
    let seconde = Uuid::new_v4();
    conn.execute(
        "INSERT INTO entreprises (id, nom, created_at, updated_at)
         VALUES (?1, 'atlas studio', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [seconde.to_string()],
    )
    .unwrap();

    repo.create(&entree(premiere, "Développeur", "2026-08-20"))
        .unwrap();
    repo.create(&entree(seconde, "Designer", "2026-08-20"))
        .unwrap();

    let page = repo
        .list_page(
            1,
            10,
            &FiltreCandidatures {
                tri: TriCandidature::Entreprise,
                ..FiltreCandidatures::default()
            },
        )
        .unwrap();

    assert_eq!(
        page.items
            .iter()
            .map(|item| item.entreprise_nom.clone().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["atlas studio", "Nova Digital"]
    );
}
