//! Cas de test isolé.

use super::*;

#[test]
fn test_est_configure_claude_sans_cle_est_faux() {
    let cfg = LlmConfig {
        provider: ProviderKind::Claude,
        api_key: None,
        endpoint: None,
        model: "claude-x".into(),
        temperature: 0.7,
        ..LlmConfig::default()
    };
    assert!(!cfg.est_configure());
}
