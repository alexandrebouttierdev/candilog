//! Cas de test isolé.

use super::*;

#[test]
fn test_resolved_mode_provider_cloud_donne_advanced() {
    let cfg = LlmConfig {
        provider: ProviderKind::Claude,
        model: "claude-sonnet-4".into(),
        ..LlmConfig::default()
    };
    assert_eq!(cfg.resolved_mode(), AnalysisMode::Advanced);
}
