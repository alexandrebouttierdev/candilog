//! Cas de test isolé.

use super::*;

#[test]
fn test_update_temperature_hors_bornes_retourne_erreur() {
    let s = AppSettings {
        llm: LlmConfig {
            temperature: 3.0,
            ..LlmConfig::default()
        },
        ..AppSettings::default()
    };
    let r = service().update(&s);
    assert!(matches!(r, Err(AppError::Validation(_))));
}
