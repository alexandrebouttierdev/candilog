//! Cas de test isolé.

use super::*;

#[test]
fn test_default_endpoint_mistral_est_api_mistral() {
    assert_eq!(
        default_endpoint(&ProviderKind::Mistral),
        "https://api.mistral.ai"
    );
}
