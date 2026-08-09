//! Cas de test isolé.

use super::*;

#[test]
fn test_update_preserve_le_contact_lie() {
    let repo = repo();
    let ent = entreprise(&repo, "ACME");
    let creee = repo.create(&entree(ent, "Dev")).unwrap();
    let contact_id = uuid::Uuid::new_v4();
    {
        let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
        conn.execute(
                "INSERT INTO contacts (id, entreprise_id, prenom, nom, created_at, updated_at)
                 VALUES (?1, ?2, 'Ada', 'Lovelace', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                rusqlite::params![contact_id.to_string(), ent.to_string()],
            )
            .unwrap();
        conn.execute(
            "UPDATE candidatures SET contact_id = ?1 WHERE id = ?2",
            rusqlite::params![contact_id.to_string(), creee.id.to_string()],
        )
        .unwrap();
    }
    let modifiee = entree(ent, "Dev Senior");
    let resultat = repo.update(creee.id, &modifiee).unwrap();
    assert_eq!(resultat.contact_id, Some(contact_id));
}
