//! Bornes du volume horaire hebdomadaire.

use super::*;

fn avec_heures(repo: &SqliteApplicationRepository, company_id: Uuid, hours: Option<f64>) {
    let mut input = entree(company_id, "Développeur", "2026-08-20");
    input.weekly_work_schedule = WeeklyWorkSchedule::PartTime;
    input.weekly_hours = hours;
    repo.create(&input).unwrap();
}

#[test]
fn les_bornes_sont_inclusives() {
    let (repo, company_id) = context();
    for hours in [17.5, 24.0, 35.0, 39.0] {
        avec_heures(&repo, company_id, Some(hours));
    }

    let page = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                min_weekly_hours: Some(24.0),
                max_weekly_hours: Some(35.0),
                ..ApplicationFilter::default()
            },
        )
        .unwrap();

    assert_eq!(page.total, 2);
}

/// « Non renseigné » n'est pas un nombre d'heures : une candidature sans volume ne peut
/// satisfaire une borne, dans un sens comme dans l'autre.
#[test]
fn une_candidature_sans_volume_horaire_sort_du_filtre_borne() {
    let (repo, company_id) = context();
    avec_heures(&repo, company_id, None);
    avec_heures(&repo, company_id, Some(35.0));

    let minimum = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                min_weekly_hours: Some(1.0),
                ..ApplicationFilter::default()
            },
        )
        .unwrap();

    assert_eq!(minimum.total, 1);
    assert_eq!(minimum.items[0].weekly_hours, Some(35.0));
}

#[test]
fn le_regime_horaire_se_filtre_independamment_du_volume() {
    let (repo, company_id) = context();
    avec_heures(&repo, company_id, Some(24.0));
    let mut plein = entree(company_id, "Développeur", "2026-08-20");
    plein.weekly_work_schedule = WeeklyWorkSchedule::FullTime;
    repo.create(&plein).unwrap();

    let page = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                weekly_work_schedule: vec![WeeklyWorkSchedule::PartTime],
                ..ApplicationFilter::default()
            },
        )
        .unwrap();

    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].weekly_hours, Some(24.0));
}
