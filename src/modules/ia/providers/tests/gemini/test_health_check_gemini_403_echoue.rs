//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_health_check_gemini_403_echoue() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", mockito::Matcher::Regex(r"^/v1beta/models".into()))
        .with_status(403)
        .create_async()
        .await;
    let p = GeminiProvider::new(server.url(), "bad".into(), "m".into(), 0.5, None);
    assert!(p.health_check().await.is_err());
}
