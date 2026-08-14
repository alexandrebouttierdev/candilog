use super::super::check_url;
use semver::Version;

#[tokio::test]
async fn check_sans_release_publiee_retourne_aucune_mise_a_jour() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/releases/latest")
        .with_status(404)
        .create_async()
        .await;
    let client = reqwest::Client::new();
    let resultat = check_url(
        &client,
        &Version::new(0, 2, 0),
        &format!("{}/releases/latest", server.url()),
    )
    .await;
    assert!(
        resultat.is_ok(),
        "un 404 ne doit pas être une erreur : {resultat:?}"
    );
    assert_eq!(resultat.unwrap(), None);
}
