use super::*;

#[test]
fn test_save_and_mark_candidate_est_atomique() {
    let repo = repo();
    let candidature_id = candidature(&repo);
    let saved = repo
        .save_and_mark_candidate(None, &entree(candidature_id, "2026-08-12T10:00:00Z"))
        .unwrap();
    assert_eq!(saved.candidature_id, candidature_id);

    let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
    let status: String = conn
        .query_row(
            "SELECT statut FROM candidatures WHERE id = ?1",
            [candidature_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "ENTRETIEN");
    let history: i64 = conn
        .query_row(
            "SELECT count(*) FROM statut_history WHERE candidature_id = ?1 AND statut = 'ENTRETIEN'",
            [candidature_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(history, 1);

    let missing = uuid::Uuid::new_v4();
    assert!(repo
        .save_and_mark_candidate(None, &entree(missing, "2026-08-13T10:00:00Z"))
        .is_err());
    let interviews: i64 = conn
        .query_row("SELECT count(*) FROM entretiens", [], |row| row.get(0))
        .unwrap();
    assert_eq!(interviews, 1, "l'entretien invalide doit être rollbacké");
}
