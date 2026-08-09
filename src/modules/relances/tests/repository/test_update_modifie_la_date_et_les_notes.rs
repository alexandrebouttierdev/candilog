//! Cas de test isolé.

use super::*;

#[test]
fn test_update_modifie_la_date_et_les_notes() {
    let repo = repo();
    let cand = candidature(&repo);
    let creee = repo.create(&entree(cand, "2026-02-01")).unwrap();
    let mut modifiee = entree(cand, "2026-02-15");
    modifiee.notes = Some("relance téléphonique".into());
    let resultat = repo.update(creee.id, &modifiee).unwrap();
    assert_eq!(resultat.date_relance, "2026-02-15");
    assert_eq!(resultat.notes.as_deref(), Some("relance téléphonique"));
}
