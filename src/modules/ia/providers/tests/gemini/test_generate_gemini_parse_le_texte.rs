//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_generate_gemini_parse_le_texte() {
    let mut server = mockito::Server::new_async().await;
    let m = server
        .mock("POST", "/v1beta/models/gemini-2.5-flash:generateContent")
        .match_header("x-goog-api-key", "key")
        .with_status(200)
        .with_body(r#"{"candidates":[{"content":{"parts":[{"text":"Salut Gemini"}]}}]}"#)
        .create_async()
        .await;
    let p = GeminiProvider::new(
        server.url(),
        "key".into(),
        "gemini-2.5-flash".into(),
        0.5,
        None,
    );
    let out = p.generate("x", "s").await.unwrap();
    assert_eq!(out, "Salut Gemini");
    m.assert_async().await;
}
