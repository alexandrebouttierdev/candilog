//! Cas de test isolé.

use super::*;

#[test]
fn test_update_custom_sans_endpoint_retourne_erreur() {
    let s = AppSettings {
        llm: LlmConfig {
            provider: ProviderKind::Custom("maison".into()),
            api_key: Some("k".into()),
            endpoint: None,
            model: "x".into(),
            temperature: 0.5,
            ..LlmConfig::default()
        },
        ..AppSettings::default()
    };
    let r = service().update(&s);
    assert!(matches!(r, Err(AppError::Validation(_))));
}
