//! Conversion des codes de durée hebdomadaire de France Travail.

use super::*;

#[test]
fn les_trois_codes_documentes_sont_traduits() {
    assert_eq!(
        WeeklyWorkSchedule::from_france_travail_code("0"),
        WeeklyWorkSchedule::Unspecified
    );
    assert_eq!(
        WeeklyWorkSchedule::from_france_travail_code("1"),
        WeeklyWorkSchedule::FullTime
    );
    assert_eq!(
        WeeklyWorkSchedule::from_france_travail_code("2"),
        WeeklyWorkSchedule::PartTime
    );
}

#[test]
fn les_espaces_du_flux_externe_sont_ignores() {
    assert_eq!(
        WeeklyWorkSchedule::from_france_travail_code(" 1 "),
        WeeklyWorkSchedule::FullTime
    );
}

#[test]
fn un_code_inconnu_vaut_non_renseignee_sans_faire_echouer_l_import() {
    for code in ["", "3", "plein", "01"] {
        assert_eq!(
            WeeklyWorkSchedule::from_france_travail_code(code),
            WeeklyWorkSchedule::Unspecified,
            "code « {code} » mal traduit"
        );
    }
}
