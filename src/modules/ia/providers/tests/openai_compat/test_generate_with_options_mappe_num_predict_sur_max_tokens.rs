//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_generate_with_options_mappe_num_predict_sur_max_tokens() {
    let mut server = mockito::Server::new_async().await;
    let m = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(
            serde_json::json!({"max_tokens": 512}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices":[{"message":{"content":"ok"}}]}"#)
        .create_async()
        .await;
    let p = OpenAiCompatProvider::new(server.url(), "sk".into(), "gpt-4o".into(), 0.5);
    let opts = GenOptions {
        num_ctx: Some(8192),
        num_predict: Some(512),
        keep_alive: None,
    };
    assert_eq!(
        p.generate_with_options("x", "s", None, &opts)
            .await
            .unwrap(),
        "ok"
    );
    m.assert_async().await;
}
