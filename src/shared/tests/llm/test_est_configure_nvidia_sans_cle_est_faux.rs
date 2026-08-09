//! Cas de test isolé.

use super::*;

#[test]
fn test_est_configure_nvidia_sans_cle_est_faux() {
    let cfg = LlmConfig {
        provider: ProviderKind::Nvidia,
        api_key: None,
        endpoint: None,
        model: "meta/llama-3.3-70b-instruct".into(),
        temperature: 0.7,
        ..LlmConfig::default()
    };
    assert!(!cfg.est_configure());
}
