//! Cas de test isolé.

use super::*;

/// Corriger l'heure d'un entretien ne doit pas rejouer le passage à l'étape entretien :
/// chaque étape en double fausserait l'entonnoir de conversion des analyses.
#[test]
fn test_enregistrer_deux_fois_n_historise_qu_une_etape() {
    let (repo, application_id) = context();
    let first = repo
        .save_and_mark_candidate(None, &entree(application_id, "2026-08-25T14:00:00+02:00"))
        .unwrap();

    repo.save_and_mark_candidate(
        Some(first.id),
        &entree(application_id, "2026-08-25T16:00:00+02:00"),
    )
    .unwrap();

    assert_eq!(steps(&repo, application_id), 1);
}
