use super::*;

#[test]
fn supprimer_retire_la_lettre_de_la_bibliotheque() {
    let repo = repo();
    let created = repo.create(&letter("À supprimer")).unwrap();
    repo.delete(created.id).unwrap();

    assert!(repo.list().unwrap().is_empty());
}
