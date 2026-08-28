//! Cas de test isolé.

use super::*;

#[test]
fn test_create_puis_list_restitue_le_contact_et_son_entreprise() {
    let repo = repo();
    let ent = company(&repo);
    let cree = repo.create(&entree("Bouttier", Some(ent))).unwrap();
    assert_eq!(cree.name, "Bouttier");
    assert_eq!(cree.company_id, Some(ent));
    assert_eq!(repo.list().unwrap().len(), 1);
}
