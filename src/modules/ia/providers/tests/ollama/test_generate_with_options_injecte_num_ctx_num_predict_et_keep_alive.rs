//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_generate_with_options_injecte_num_ctx_num_predict_et_keep_alive() {
    let mut server = mockito::Server::new_async().await;
    let m = server
        .mock("POST", "/api/chat")
        .match_body(mockito::Matcher::PartialJson(serde_json::json!({
            "keep_alive": "10m",
            "options": {"num_ctx": 4096, "num_predict": 256}
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":{"content":"{}"}}"#)
        .create_async()
        .await;
    let p = OllamaProvider::new(server.url(), "gemma3:1b".into(), 0.0, None);
    let opts = GenOptions {
        num_ctx: Some(4096),
        num_predict: Some(256),
        keep_alive: Some("10m"),
    };
    let out = p
        .generate_with_options("cv", "sys", None, &opts)
        .await
        .unwrap();
    assert_eq!(out, "{}");
    m.assert_async().await;
}
