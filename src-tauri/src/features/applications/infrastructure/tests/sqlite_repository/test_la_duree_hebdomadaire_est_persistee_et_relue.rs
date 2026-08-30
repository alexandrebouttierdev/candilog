//! Régime horaire et volume hebdomadaire.

use super::*;

#[test]
fn les_trois_regimes_et_les_volumes_decimaux_sont_relus_a_l_identique() {
    let (repo, company_id) = context();

    for (schedule, hours) in [
        (WeeklyWorkSchedule::FullTime, Some(35.0)),
        (WeeklyWorkSchedule::FullTime, Some(39.0)),
        (WeeklyWorkSchedule::PartTime, Some(24.0)),
        (WeeklyWorkSchedule::PartTime, Some(17.5)),
        (WeeklyWorkSchedule::Unspecified, None),
        (WeeklyWorkSchedule::FullTime, None),
    ] {
        let mut input = entree(company_id, "Développeur", "2026-08-20");
        input.weekly_work_schedule = schedule;
        input.weekly_hours = hours;

        let creee = repo.create(&input).unwrap();

        assert_eq!(creee.weekly_work_schedule, schedule);
        assert_eq!(creee.weekly_hours, hours);
    }
}

/// Le `CHECK` du schéma est la dernière barrière : le service valide déjà les bornes, mais
/// une écriture qui le contournerait ne doit pas passer.
#[test]
fn la_base_refuse_un_volume_horaire_aberrant() {
    let (repo, company_id) = context();
    let creee = repo
        .create(&entree(company_id, "Développeur", "2026-08-20"))
        .unwrap();

    for hours in ["0", "-1", "200"] {
        let resultat = connection(&repo.pool).unwrap().execute(
            &format!("UPDATE applications SET weekly_hours = {hours} WHERE id = ?1"),
            [creee.id.to_string()],
        );
        assert!(resultat.is_err(), "{hours} h/semaine accepté par la base");
    }
}
