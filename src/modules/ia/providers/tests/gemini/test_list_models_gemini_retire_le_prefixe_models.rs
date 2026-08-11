//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_list_models_gemini_retire_le_prefixe_models() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", mockito::Matcher::Regex(r"^/v1beta/models".into()))
        .with_status(200)
        .with_body(
            r#"{"models":[{"name":"models/gemini-2.5-flash"},{"name":"models/gemini-2.5-pro"}]}"#,
        )
        .create_async()
        .await;
    let p = GeminiProvider::new(server.url(), "key".into(), "m".into(), 0.5, None);
    let models = p.list_models().await.unwrap();
    assert_eq!(models, vec!["gemini-2.5-flash", "gemini-2.5-pro"]);
}
