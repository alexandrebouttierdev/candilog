//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_stream_ollama_accumule_les_fragments_ndjson() {
    let mut server = mockito::Server::new_async().await;
    let body = "{\"message\":{\"content\":\"Bonjour \"},\"done\":false}\n{\"message\":{\"content\":\"le monde\"},\"done\":true}\n";
    let m = server
        .mock("POST", "/api/chat")
        .match_body(mockito::Matcher::PartialJson(
            serde_json::json!({"stream": true}),
        ))
        .with_status(200)
        .with_body(body)
        .create_async()
        .await;
    let p = OllamaProvider::new(server.url(), "gemma3:1b".into(), 0.0);
    let mut chunks: Vec<String> = Vec::new();
    let full = p
        .stream("cv", "sys", &GenOptions::none(), &mut |c: String| {
            chunks.push(c)
        })
        .await
        .unwrap();
    assert_eq!(full, "Bonjour le monde");
    assert_eq!(chunks, vec!["Bonjour ", "le monde"]);
    m.assert_async().await;
}
