//! Cas de test isolé.

use super::*;

/// Planifier un entretien fait avancer la candidature : sans cela, une candidature
/// resterait « en attente » alors qu'un entretien est déjà au calendrier.
#[test]
fn test_enregistrer_fait_avancer_la_candidature() {
    let (repo, candidature_id) = contexte();
    assert_eq!(statut(&repo, candidature_id), "EN_ATTENTE");

    repo.save_and_mark_candidate(None, &entree(candidature_id, "2026-08-25T14:00:00+02:00"))
        .unwrap();

    assert_eq!(statut(&repo, candidature_id), "ENTRETIEN");
    assert_eq!(etapes(&repo, candidature_id), 1);
}
