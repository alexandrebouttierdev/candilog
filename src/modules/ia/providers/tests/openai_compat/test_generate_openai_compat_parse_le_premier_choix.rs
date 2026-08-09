//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_generate_openai_compat_parse_le_premier_choix() {
    let mut server = mockito::Server::new_async().await;
    let m = server
        .mock("POST", "/v1/chat/completions")
        .match_header("authorization", "Bearer sk-test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices":[{"message":{"content":"Salut"}}]}"#)
        .create_async()
        .await;
    let p = OpenAiCompatProvider::new(server.url(), "sk-test".into(), "gpt-4o".into(), 0.5);
    let out = p.generate("x", "s").await.unwrap();
    assert_eq!(out, "Salut");
    m.assert_async().await;
}
