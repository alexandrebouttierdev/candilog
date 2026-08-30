//! Cas de test isolé.

use super::*;

/// La recherche et le filtre par rôle comparaient tous deux une valeur normalisée en Rust à
/// une colonne passée par `lower()` de SQLite, qui laisse les majuscules accentuées intactes.
#[test]
fn test_la_recherche_et_le_role_ignorent_les_accents() {
    let repo = repo();
    let mut contact = entree("ÉMERY", None);
    contact.tracking_role = Some("Décideur".into());
    repo.create(&contact).unwrap();

    for terme in ["émery", "EMERY", "Emery"] {
        let page = repo.list_page(1, 10, terme, None).unwrap();
        assert_eq!(page.total, 1, "recherche « {terme} » sans résultat");
    }
    let page = repo.list_page(1, 10, "", Some("DÉCIDEUR")).unwrap();
    assert_eq!(page.total, 1, "filtre par rôle accentué sans résultat");
}
