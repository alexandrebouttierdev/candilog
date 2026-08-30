//! Les détails réseau peuvent contenir une URL, ses paramètres et donc un secret.

use super::*;

#[test]
fn erreur_http_masque_url_et_secret_dans_le_contrat_ipc() {
    let error = AppError::Http(
        "error sending request for url (https://api.example.test/v1?api_key=sk-secret)".into(),
    );

    let json = serde_json::to_value(error).unwrap().to_string();

    assert!(!json.contains("api.example.test"));
    assert!(!json.contains("api_key"));
    assert!(!json.contains("sk-secret"));
    assert!(json.contains("HTTP_ERROR"));
}
