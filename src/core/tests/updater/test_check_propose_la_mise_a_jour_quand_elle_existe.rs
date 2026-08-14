use super::super::check_url;
use semver::Version;

#[tokio::test]
async fn check_propose_la_mise_a_jour_quand_elle_existe() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/releases/latest")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "tag_name": "v0.3.0",
                "body": "Notes de la version.",
                "html_url": "https://example.test/releases/tag/v0.3.0",
                "assets": []
            }"#,
        )
        .create_async()
        .await;
    let client = reqwest::Client::new();
    let resultat = check_url(
        &client,
        &Version::new(0, 2, 0),
        &format!("{}/releases/latest", server.url()),
    )
    .await;
    let info = resultat
        .expect("la vérification doit réussir")
        .expect("une version plus récente doit être proposée");
    assert_eq!(info.version, Version::new(0, 3, 0));
    assert_eq!(info.notes, "Notes de la version.");
}
