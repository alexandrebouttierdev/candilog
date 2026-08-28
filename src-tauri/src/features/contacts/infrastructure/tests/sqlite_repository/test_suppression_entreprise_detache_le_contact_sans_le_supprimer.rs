//! Cas de test isolé.

use super::*;

#[test]
fn test_suppression_entreprise_detache_le_contact_sans_le_supprimer() {
    let repo = repo();
    let ent = company(&repo);
    repo.create(&entree("Bouttier", Some(ent))).unwrap();
    {
        let conn = crate::core::database::helpers::connection(&repo.pool).unwrap();
        conn.execute("DELETE FROM companies WHERE id = ?1", [ent.to_string()])
            .unwrap();
    }
    let contacts = repo.list().unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].company_id, None);
}
