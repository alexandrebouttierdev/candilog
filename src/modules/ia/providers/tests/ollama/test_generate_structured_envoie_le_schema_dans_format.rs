//! Vérifie l'envoi du schéma JSON au provider Ollama.

use super::*;

#[tokio::test]
async fn test_generate_structured_envoie_le_schema_dans_format() {
    let mut server = mockito::Server::new_async().await;
    let schema = serde_json::json!({
        "type": "object",
        "properties": {"name": {"type": "string"}},
        "required": ["name"]
    });
    let m = server
        .mock("POST", "/api/chat")
        .match_body(mockito::Matcher::PartialJson(serde_json::json!({
            "format": schema.clone()
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":{"content":"{\"name\":\"Ada\"}"}}"#)
        .create_async()
        .await;
    let p = OllamaProvider::new(server.url(), "gemma3:1b".into(), 0.0, None);
    let out = p.generate_structured("cv", "sys", &schema).await.unwrap();
    assert_eq!(out, r#"{"name":"Ada"}"#);
    m.assert_async().await;
}
