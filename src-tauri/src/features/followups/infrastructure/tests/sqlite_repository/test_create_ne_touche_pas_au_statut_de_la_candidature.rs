//! Cas de test isolé.

use super::*;

/// Comportement repris de l'application Iced : contrairement à l'enregistrement d'un
/// entretien, créer une relance ne fait pas avancer la candidature. Ce test fige
/// l'asymétrie pour qu'un changement de comportement soit délibéré et non accidentel.
#[test]
fn test_create_ne_touche_pas_au_statut_de_la_candidature() {
    let (repo, application_id) = context();

    repo.create(&entree(application_id, "2026-08-27")).unwrap();

    assert_eq!(status(&repo, application_id), "EN_ATTENTE");
}
