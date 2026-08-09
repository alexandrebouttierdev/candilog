//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_health_check_claude_401_echoue() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/models")
        .with_status(401)
        .create_async()
        .await;
    let p = ClaudeProvider::new(server.url(), "bad".into(), "m".into(), 0.5);
    assert!(p.health_check().await.is_err());
}
