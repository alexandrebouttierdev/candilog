//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_contact_lie_a_un_entretien_seul_est_refuse() {
    // Cas que le frontend ne couvrait pas : il ne grise le bouton que si des candidatures
    // sont liées, en s'appuyant sur un refus backend qui doit donc exister.
    let repo = repo();
    let cree = repo.create(&entree("Bouttier", None)).unwrap();
    let application_id = application_liee(&repo, cree.id);
    {
        let conn = crate::core::database::helpers::connection(&repo.pool).unwrap();
        // On détache la candidature : seul l'entretien référence encore le contact.
        conn.execute(
            "UPDATE applications SET contact_id = NULL WHERE id = ?1",
            [application_id.to_string()],
        )
        .unwrap();
        conn.execute(
                "INSERT INTO interviews (id, application_id, contact_id, interview_date, created_at, updated_at)
                 VALUES (?1, ?2, ?3, '2026-02-01T10:00:00Z', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    application_id.to_string(),
                    cree.id.to_string()
                ],
            )
            .unwrap();
    }
    let resultat = repo.delete(cree.id);
    assert!(matches!(resultat, Err(AppError::Validation(_))));
    assert_eq!(repo.list().unwrap().len(), 1);
}
