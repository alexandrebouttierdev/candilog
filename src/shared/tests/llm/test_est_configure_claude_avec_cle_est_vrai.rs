//! Cas de test isolé.

use super::*;

#[test]
fn test_est_configure_claude_avec_cle_est_vrai() {
    let cfg = LlmConfig {
        provider: ProviderKind::Claude,
        api_key: Some("sk-xxx".into()),
        endpoint: None,
        model: "claude-x".into(),
        temperature: 0.7,
        ..LlmConfig::default()
    };
    assert!(cfg.est_configure());
}
