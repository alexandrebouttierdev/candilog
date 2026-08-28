//! Cas de test isolé.

use super::*;

/// Planifier un entretien fait avancer la candidature : sans cela, une candidature
/// resterait « en attente » alors qu'un entretien est déjà au calendrier.
#[test]
fn test_enregistrer_fait_avancer_la_candidature() {
    let (repo, application_id) = context();
    assert_eq!(status(&repo, application_id), "EN_ATTENTE");

    repo.save_and_mark_candidate(None, &entree(application_id, "2026-08-25T14:00:00+02:00"))
        .unwrap();

    assert_eq!(status(&repo, application_id), "ENTRETIEN");
    assert_eq!(steps(&repo, application_id), 1);
}
