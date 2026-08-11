//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_list_models_ollama_extrait_les_noms() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/api/tags")
        .with_status(200)
        .with_body(r#"{"models":[{"name":"llama3.2:3b"},{"name":"qwen2.5:7b"}]}"#)
        .create_async()
        .await;
    let p = OllamaProvider::new(server.url(), "m".into(), 0.7, None);
    let models = p.list_models().await.unwrap();
    assert_eq!(models, vec!["llama3.2:3b", "qwen2.5:7b"]);
}
