//! Contrat public d'une base issue d'une version abandonnée.

use super::*;

#[test]
fn donnees_incompatibles_exposent_un_message_actionnable() {
    let error = AppError::IncompatibleData("version 9".into());

    assert_eq!(error.code(), "INCOMPATIBLE_DATA");
    assert!(error.user_message().contains("ancienne version"));
    assert!(!error.user_message().contains("version 9"));
}
