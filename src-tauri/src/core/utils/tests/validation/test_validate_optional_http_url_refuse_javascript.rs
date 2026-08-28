//! Cas de test isolé.

use super::*;

#[test]
fn test_validate_optional_http_url_refuse_javascript() {
    assert!(validate_optional_http_url(Some("javascript:alert(1)"), "URL").is_err());
}
