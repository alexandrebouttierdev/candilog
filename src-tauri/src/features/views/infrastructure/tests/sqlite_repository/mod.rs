//! Vues enregistrées sur une base mémoire migrée.

use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use crate::features::applications::domain::{ApplicationFilter, ApplicationStatus};
use crate::features::views::application::SavedViewService;

fn service() -> SavedViewService<SqliteSavedViewRepository> {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SavedViewService::new(SqliteSavedViewRepository::new(pool))
}

fn vue(name: &str) -> NewSavedView {
    NewSavedView {
        name: name.into(),
        filter: ApplicationFilter {
            status: vec![ApplicationStatus::Pending, ApplicationStatus::FollowedUp],
            search: "rust".into(),
            ..ApplicationFilter::default()
        },
    }
}

#[test]
fn une_vue_garde_son_filtre_et_prend_la_derniere_position() {
    let service = service();
    let premiere = service.create(&vue("À relancer")).unwrap();
    let seconde = service.create(&vue("Entretiens à venir")).unwrap();

    assert!(seconde.position > premiere.position);
    let relue = &service.list().unwrap()[0];
    assert_eq!(relue.name, "À relancer");
    assert_eq!(
        relue.filter.status,
        vec![ApplicationStatus::Pending, ApplicationStatus::FollowedUp]
    );
    assert_eq!(relue.filter.search, "rust");
}

#[test]
fn le_nom_est_obligatoire_et_borne() {
    let service = service();
    assert!(matches!(
        service.create(&vue("   ")),
        Err(AppError::Validation(_))
    ));
    assert!(matches!(
        service.create(&vue(&"x".repeat(61))),
        Err(AppError::Validation(_))
    ));
}

#[test]
fn renommer_dupliquer_et_supprimer() {
    let service = service();
    let vue_creee = service.create(&vue("Spontanées")).unwrap();

    let renommee = service
        .update(vue_creee.id, &vue("Spontanées 2026"))
        .unwrap();
    assert_eq!(renommee.name, "Spontanées 2026");

    let copie = service.duplicate(vue_creee.id).unwrap();
    assert_eq!(copie.name, "Spontanées 2026 (copie)");
    assert_eq!(copie.filter.search, "rust");

    service.delete(vue_creee.id).unwrap();
    assert_eq!(service.list().unwrap().len(), 1);
    assert!(matches!(
        service.delete(vue_creee.id),
        Err(AppError::NotFound(_))
    ));
}
