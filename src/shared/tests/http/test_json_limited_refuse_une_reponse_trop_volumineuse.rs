//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_json_limited_refuse_une_reponse_trop_volumineuse() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/large")
        .with_status(200)
        .with_body("123456")
        .create_async()
        .await;
    let response = client()
        .get(format!("{}/large", server.url()))
        .send()
        .await
        .unwrap();
    assert!(json_limited::<serde_json::Value>(response, 5)
        .await
        .is_err());
}
