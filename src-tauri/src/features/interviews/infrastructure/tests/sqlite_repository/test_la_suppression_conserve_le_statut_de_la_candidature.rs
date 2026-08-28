//! Cas de test isolé.

use super::*;

/// Delete un entretien annulé ne veut pas dire que la candidature n'a jamais atteint
/// cette étape : rétrograder le statut effacerait une information vraie.
#[test]
fn test_la_suppression_conserve_le_statut_de_la_candidature() {
    let (repo, application_id) = context();
    let cree = repo
        .save_and_mark_candidate(None, &entree(application_id, "2026-08-25T14:00:00+02:00"))
        .unwrap();

    repo.delete(cree.id).unwrap();

    assert_eq!(status(&repo, application_id), "ENTRETIEN");
    assert!(matches!(repo.get(cree.id), Err(AppError::NotFound(_))));
}
