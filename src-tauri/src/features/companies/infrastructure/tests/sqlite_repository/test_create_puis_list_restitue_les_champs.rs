//! Cas de test isolé.

use super::*;

#[test]
fn test_create_puis_list_restitue_les_champs() {
    let repo = repo();
    let creee = repo.create(&entree("ACME")).unwrap();
    assert_eq!(creee.name, "ACME");
    assert_eq!(creee.city.as_deref(), Some("Lyon"));
    assert_eq!(creee.company_size, CompanySize::Pme);
    assert_eq!(
        creee.company_type_name.as_deref(),
        Some("ESN / Société de services numériques")
    );
    assert!(!creee.created_at.is_empty());

    let toutes = repo.list().unwrap();
    assert_eq!(toutes.len(), 1);
    assert_eq!(toutes[0].id, creee.id);
}

/// Une entreprise dont la taille n'est pas renseignée porte `UNKNOWN`, jamais `NULL` : une
/// seule représentation du vide à filtrer et à afficher.
#[test]
fn une_taille_absente_vaut_non_renseignee() {
    let repo = repo();
    let mut sans_taille = entree("ACME");
    sans_taille.company_size = CompanySize::Unknown;

    let creee = repo.create(&sans_taille).unwrap();

    assert_eq!(creee.company_size, CompanySize::Unknown);
}
