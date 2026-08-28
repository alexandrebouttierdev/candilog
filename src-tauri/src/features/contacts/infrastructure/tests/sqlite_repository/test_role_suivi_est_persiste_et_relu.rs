//! Cas de test isolé.

use super::*;

/// Le rôle dans le suivi est la seule colonne introduite par la migration 009 : elle n'est
/// couverte par aucun test hérité, et une faute de frappe dans l'INSERT ou dans la liste de
/// colonnes du SELECT passerait autrement inaperçue jusqu'à l'écran.
#[test]
fn test_role_suivi_est_persiste_et_relu() {
    let repo = repo();
    let cree = repo.create(&entree("Rivet", None)).unwrap();
    assert_eq!(cree.role_suivi.as_deref(), Some("Manager"));

    let mut modifie = entree("Rivet", None);
    modifie.role_suivi = Some("Recruteur".into());
    let apres = repo.update(cree.id, &modifie).unwrap();

    assert_eq!(apres.role_suivi.as_deref(), Some("Recruteur"));
    assert_eq!(
        repo.get(cree.id).unwrap().role_suivi.as_deref(),
        Some("Recruteur")
    );
}
