use super::*;

#[test]
fn enregistrer_et_recharger_restitue_le_document() {
    let repo = repo();
    let created = repo.create(&letter("Candidature Rust")).unwrap();
    let loaded = repo.get(created.id).unwrap();

    assert_eq!(loaded.name, "Candidature Rust");
    assert_eq!(loaded.company.as_deref(), Some("Candilog"));
    assert_eq!(repo.list().unwrap().len(), 1);
}
