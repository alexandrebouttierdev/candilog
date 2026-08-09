//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_validate_endpoint_cloud_refuse_une_adresse_privee() {
    let config = LlmConfig {
        provider: ProviderKind::OpenAI,
        api_key: Some("key".into()),
        endpoint: Some("https://127.0.0.1/v1".into()),
        model: "model".into(),
        temperature: 0.5,
        ..LlmConfig::default()
    };
    assert!(validate_llm_endpoint(&config).await.is_err());
}
