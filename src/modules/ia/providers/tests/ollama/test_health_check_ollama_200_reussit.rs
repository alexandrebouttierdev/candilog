//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_health_check_ollama_200_reussit() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_body("{}")
        .create_async()
        .await;
    let p = OllamaProvider::new(server.url(), "m".into(), 0.7);
    assert!(p.health_check().await.is_ok());
}
