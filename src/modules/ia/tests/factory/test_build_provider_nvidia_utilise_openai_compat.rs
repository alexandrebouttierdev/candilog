//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_build_provider_nvidia_utilise_openai_compat() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/models")
        .with_status(200)
        .with_body(r#"{"data":[{"id":"test-model"}]}"#)
        .create_async()
        .await;
    let config = LlmConfig {
        provider: ProviderKind::Nvidia,
        api_key: Some("nvapi-key".into()),
        endpoint: Some(server.url()),
        model: "meta/llama-3.3-70b-instruct".into(),
        temperature: 0.7,
        ..LlmConfig::default()
    };
    let provider = build_provider(&config);
    assert!(provider.health_check().await.is_ok());
}
