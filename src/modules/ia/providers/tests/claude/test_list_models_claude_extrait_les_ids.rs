//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_list_models_claude_extrait_les_ids() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/models")
        .with_status(200)
        .with_body(r#"{"data":[{"id":"claude-sonnet-4"},{"id":"claude-opus-4"}]}"#)
        .create_async()
        .await;
    let p = ClaudeProvider::new(server.url(), "sk-ant".into(), "m".into(), 0.5);
    let models = p.list_models().await.unwrap();
    assert_eq!(models, vec!["claude-sonnet-4", "claude-opus-4"]);
}
