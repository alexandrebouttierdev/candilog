//! Cas de test isolé.

use super::*;

#[test]
fn test_validate_optional_http_url_accepte_https() {
    assert!(validate_optional_http_url(Some("https://example.com/path"), "URL").is_ok());
}
