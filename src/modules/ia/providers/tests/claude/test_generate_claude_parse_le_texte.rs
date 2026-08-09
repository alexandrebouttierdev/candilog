//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_generate_claude_parse_le_texte() {
    let mut server = mockito::Server::new_async().await;
    let m = server
        .mock("POST", "/v1/messages")
        .match_header("x-api-key", "sk-ant")
        .with_status(200)
        .with_body(r#"{"content":[{"type":"text","text":"Réponse"}]}"#)
        .create_async()
        .await;
    let p = ClaudeProvider::new(server.url(), "sk-ant".into(), "claude-sonnet-4".into(), 0.5);
    let out = p.generate("x", "s").await.unwrap();
    assert_eq!(out, "Réponse");
    m.assert_async().await;
}
