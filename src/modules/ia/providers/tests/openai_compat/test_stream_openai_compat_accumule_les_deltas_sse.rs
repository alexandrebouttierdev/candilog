//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_stream_openai_compat_accumule_les_deltas_sse() {
    let mut server = mockito::Server::new_async().await;
    let body = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello \"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"world\"}}]}\n\ndata: [DONE]\n\n";
    let m = server
        .mock("POST", "/v1/chat/completions")
        .match_body(mockito::Matcher::PartialJson(
            serde_json::json!({"stream": true}),
        ))
        .with_status(200)
        .with_body(body)
        .create_async()
        .await;
    let p = OpenAiCompatProvider::new(server.url(), "sk".into(), "gpt-4o".into(), 0.5, None);
    let mut chunks: Vec<String> = Vec::new();
    let full = p
        .stream("x", "s", &GenOptions::none(), &mut |c: String| {
            chunks.push(c)
        })
        .await
        .unwrap();
    assert_eq!(full, "Hello world");
    assert_eq!(chunks, vec!["Hello ", "world"]);
    m.assert_async().await;
}
