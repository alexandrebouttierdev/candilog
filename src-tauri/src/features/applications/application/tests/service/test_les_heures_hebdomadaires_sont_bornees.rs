//! Le volume horaire hebdomadaire doit rester un nombre d'heures plausible.

use super::*;
use crate::features::applications::domain::MAX_WEEKLY_HOURS;

#[test]
fn les_volumes_courants_sont_acceptes() {
    let service = ApplicationService::new(StubRepo::default());

    for hours in [35.0, 39.0, 24.0, 17.5, 0.5, MAX_WEEKLY_HOURS] {
        let mut input = new("Développeur");
        input.weekly_hours = Some(hours);
        assert!(
            service.create(&input).is_ok(),
            "{hours} h/semaine aurait dû être accepté"
        );
    }
}

#[test]
fn l_absence_de_volume_reste_valide() {
    let service = ApplicationService::new(StubRepo::default());
    let mut input = new("Développeur");
    input.weekly_hours = None;

    assert!(service.create(&input).is_ok());
}

/// `NaN` et l'infini traversent `f64` et JSON sans encombre, et toute comparaison avec
/// `NaN` est fausse : une borne haute seule les laisserait entrer en base.
#[test]
fn les_valeurs_aberrantes_sont_refusees() {
    let service = ApplicationService::new(StubRepo::default());

    for hours in [
        0.0,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        MAX_WEEKLY_HOURS + 0.5,
        400.0,
    ] {
        let mut input = new("Développeur");
        input.weekly_hours = Some(hours);
        assert!(
            matches!(service.create(&input), Err(AppError::Validation(_))),
            "{hours} h/semaine aurait dû être refusé"
        );
    }
}
