//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_list_models_openai_compat_extrait_les_ids() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/models")
        .with_status(200)
        .with_body(r#"{"data":[{"id":"gpt-4o"},{"id":"gpt-4o-mini"}]}"#)
        .create_async()
        .await;
    let p = OpenAiCompatProvider::new(server.url(), "sk".into(), "m".into(), 0.5, None);
    let models = p.list_models().await.unwrap();
    assert_eq!(models, vec!["gpt-4o", "gpt-4o-mini"]);
}
