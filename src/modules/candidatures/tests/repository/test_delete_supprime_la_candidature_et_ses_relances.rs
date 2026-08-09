//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_supprime_la_candidature_et_ses_relances() {
    let repo = repo();
    let ent = entreprise(&repo, "ACME");
    let creee = repo.create(&entree(ent, "Dev")).unwrap();
    {
        let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
        conn.execute(
            "INSERT INTO relances (id, candidature_id, date_relance, created_at)
                 VALUES (?1, ?2, '2026-02-01', '2026-02-01T00:00:00Z')",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), creee.id.to_string()],
        )
        .unwrap();
    }
    repo.delete(creee.id).unwrap();
    let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
    let relances: i64 = conn
        .query_row("SELECT count(*) FROM relances", [], |r| r.get(0))
        .unwrap();
    assert!(repo.list().unwrap().is_empty());
    assert_eq!(relances, 0);
}
