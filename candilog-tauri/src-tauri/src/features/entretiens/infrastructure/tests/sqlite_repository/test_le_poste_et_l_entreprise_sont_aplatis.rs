//! Cas de test isolé.

use super::*;

/// Le calendrier affiche « Développeur Frontend — Nova Digital » sur chaque pastille : sans
/// aplatissement, il faudrait une requête par événement.
#[test]
fn test_le_poste_et_l_entreprise_sont_aplatis() {
    let (repo, candidature_id) = contexte();
    let cree = repo
        .save_and_mark_candidate(None, &entree(candidature_id, "2026-08-25T14:00:00+02:00"))
        .unwrap();

    assert_eq!(
        cree.candidature_poste.as_deref(),
        Some("Développeur Frontend")
    );
    assert_eq!(cree.entreprise_nom.as_deref(), Some("Nova Digital"));
}
