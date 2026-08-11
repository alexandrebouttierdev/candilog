//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_chat_modele_raisonnement_envoie_think_false() {
    let mut server = mockito::Server::new_async().await;
    let m = server
        .mock("POST", "/api/chat")
        .match_body(mockito::Matcher::PartialJson(
            serde_json::json!({"think": false}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":{"content":"{}"}}"#)
        .create_async()
        .await;
    let p = OllamaProvider::new(server.url(), "gpt-oss:20b".into(), 0.0, None);
    assert_eq!(p.generate("cv", "sys").await.unwrap(), "{}");
    m.assert_async().await;
}
