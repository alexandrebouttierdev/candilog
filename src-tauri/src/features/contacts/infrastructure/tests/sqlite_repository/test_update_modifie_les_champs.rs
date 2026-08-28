//! Cas de test isolé.

use super::*;

#[test]
fn test_update_modifie_les_champs() {
    let repo = repo();
    let cree = repo.create(&entree("Bouttier", None)).unwrap();
    let mut modifie = entree("Bouttier", None);
    modifie.job_title = Some("CEO".into());
    let resultat = repo.update(cree.id, &modifie).unwrap();
    assert_eq!(resultat.job_title.as_deref(), Some("CEO"));
}
