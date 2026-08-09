//! Cas de test isolé.

use super::*;

#[test]
fn test_update_provider_cloud_sans_cle_retourne_erreur() {
    let s = AppSettings {
        llm: LlmConfig {
            provider: ProviderKind::OpenAI,
            api_key: None,
            endpoint: None,
            model: "gpt-4o".into(),
            temperature: 0.5,
            ..LlmConfig::default()
        },
        ..AppSettings::default()
    };
    let r = service().update(&s);
    assert!(matches!(r, Err(AppError::Validation(_))));
}
