//! Cas de test isolé.

use super::*;

/// Le répertoire des entreprises souffrait du même écart que le suivi : la normalisation
/// Unicode du terme recherché ne rencontrait jamais la majuscule accentuée stockée, que
/// `lower()` de SQLite laisse intacte.
#[test]
fn test_la_recherche_ignore_les_accents_et_la_casse() {
    let repo = repo();
    repo.create(&entree("ÉCOLE DIRECTE")).unwrap();

    for terme in ["école", "ECOLE", "Écolé"] {
        let page = repo.list_page(1, 10, &recherche(terme)).unwrap();
        assert_eq!(page.total, 1, "recherche « {terme} » sans résultat");
    }
}
