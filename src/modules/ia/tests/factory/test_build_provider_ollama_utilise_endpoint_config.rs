//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_build_provider_ollama_utilise_endpoint_config() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_body("{}")
        .create_async()
        .await;
    let config = LlmConfig {
        provider: ProviderKind::Ollama,
        api_key: None,
        endpoint: Some(server.url()),
        model: "m".into(),
        temperature: 0.7,
        ..LlmConfig::default()
    };
    let provider = build_provider(&config);
    assert!(provider.health_check().await.is_ok());
}
