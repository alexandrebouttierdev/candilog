//! Cas de test isolé.

use super::*;

#[test]
fn test_update_modifie_les_champs_et_l_horodatage() {
    let repo = repo();
    let creee = repo.create(&entree("ACME")).unwrap();
    let mut modifiee = entree("ACME renommée");
    modifiee.notes = Some("client historique".into());
    let resultat = repo.update(creee.id, &modifiee).unwrap();
    assert_eq!(resultat.nom, "ACME renommée");
    assert_eq!(resultat.notes.as_deref(), Some("client historique"));
    assert_eq!(resultat.id, creee.id);
}
