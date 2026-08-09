//! Cas de test isolé.

use super::*;

#[test]
fn test_default_endpoint_custom_utilise_base_openai() {
    assert_eq!(
        default_endpoint(&ProviderKind::Custom("x".into())),
        "https://api.openai.com"
    );
}
