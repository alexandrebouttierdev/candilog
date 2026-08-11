//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_generate_ollama_parse_le_contenu() {
    let mut server = mockito::Server::new_async().await;
    let m = server
        .mock("POST", "/api/chat")
        .match_body(mockito::Matcher::PartialJson(serde_json::json!({
            "model": "llama3.2:3b",
            "stream": false,
            "format": "json"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":{"content":"Bonjour"}}"#)
        .create_async()
        .await;
    let p = OllamaProvider::new(server.url(), "llama3.2:3b".into(), 0.7, None);
    let out = p.generate("salut", "sys").await.unwrap();
    assert_eq!(out, "Bonjour");
    m.assert_async().await;
}
