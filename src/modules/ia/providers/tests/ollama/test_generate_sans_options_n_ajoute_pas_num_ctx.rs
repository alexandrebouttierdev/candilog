//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_generate_sans_options_n_ajoute_pas_num_ctx() {
    let mut server = mockito::Server::new_async().await;
    // `generate` classique : `options` ne contient que la température (pas de num_ctx).
    let m = server
        .mock("POST", "/api/chat")
        .match_body(mockito::Matcher::PartialJson(
            serde_json::json!({"options": {"temperature": 0.0}}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":{"content":"ok"}}"#)
        .create_async()
        .await;
    let p = OllamaProvider::new(server.url(), "gemma3:1b".into(), 0.0);
    assert_eq!(p.generate("x", "s").await.unwrap(), "ok");
    m.assert_async().await;
}
