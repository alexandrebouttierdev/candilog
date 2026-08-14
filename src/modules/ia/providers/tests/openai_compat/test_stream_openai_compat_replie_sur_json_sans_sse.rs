//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_stream_openai_compat_replie_sur_json_sans_sse() {
    let mut server = mockito::Server::new_async().await;
    let body = r#"{"choices":[{"message":{"content":"Lettre complète"}}]}"#;
    let m = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(
            serde_json::json!({"stream": true}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;
    let p = OpenAiCompatProvider::new(server.url(), "sk".into(), "gpt-4o".into(), 0.5, None);
    let mut morceaux: Vec<String> = Vec::new();
    let complete = p
        .stream("x", "s", &GenOptions::none(), &mut |c: String| {
            morceaux.push(c)
        })
        .await
        .unwrap();
    assert_eq!(complete, "Lettre complète");
    assert_eq!(morceaux, vec!["Lettre complète"]);
    m.assert_async().await;
}
