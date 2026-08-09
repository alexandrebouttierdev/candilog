//! Cas de test isolé.

use super::*;

#[test]
fn test_update_modifie_le_compte_rendu() {
    let repo = repo();
    let cand = candidature(&repo);
    let cree = repo.create(&entree(cand, "2026-03-01T10:00:00Z")).unwrap();
    let mut modifie = entree(cand, "2026-03-01T10:00:00Z");
    modifie.compte_rendu = Some("Entretien technique de 45 min".into());
    let resultat = repo.update(cree.id, &modifie).unwrap();
    assert_eq!(
        resultat.compte_rendu.as_deref(),
        Some("Entretien technique de 45 min")
    );
}
