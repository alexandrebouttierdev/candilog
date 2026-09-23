//! Relance faite (« Faire » dans Aujourd'hui) et son annulation.

use super::*;

#[test]
fn une_relance_faite_porte_son_horodatage_et_peut_etre_rouverte() {
    let (repo, application_id) = context();
    let relance = repo.create(&entree(application_id, "2026-08-20")).unwrap();
    assert_eq!(relance.done_at, None);

    let faite = repo.set_done(relance.id, true).unwrap();
    assert!(faite.done_at.is_some());

    // « annuler » à l'endroit du geste : la relance redevient à faire.
    let rouverte = repo.set_done(relance.id, false).unwrap();
    assert_eq!(rouverte.done_at, None);
}

#[test]
fn marquer_une_relance_inconnue_retourne_not_found() {
    let (repo, _) = context();
    assert!(matches!(
        repo.set_done(Uuid::new_v4(), true),
        Err(AppError::NotFound(_))
    ));
}
