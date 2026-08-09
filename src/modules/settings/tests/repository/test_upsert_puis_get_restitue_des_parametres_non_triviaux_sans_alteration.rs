//! Cas de test isolé.

use super::*;

#[test]
fn test_upsert_puis_get_restitue_des_parametres_non_triviaux_sans_alteration() {
    use crate::modules::settings::model::ThemePref;
    use crate::shared::llm::{LlmConfig, ProviderKind};

    let repo = repo();
    let parametres = AppSettings {
        llm: LlmConfig {
            provider: ProviderKind::OpenAI,
            api_key: Some("sk-test-1234".into()),
            endpoint: Some("https://api.openai.com/v1".into()),
            model: "gpt-4o".into(),
            temperature: 0.9,
            ..LlmConfig::default()
        },
        theme: ThemePref::Dark,
        langue: "en".into(),
    };
    repo.upsert(&parametres).unwrap();
    assert_eq!(repo.get().unwrap(), parametres);
}
