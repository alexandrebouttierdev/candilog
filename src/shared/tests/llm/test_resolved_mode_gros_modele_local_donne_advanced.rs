//! Cas de test isolé.

use super::*;

#[test]
fn test_resolved_mode_gros_modele_local_donne_advanced() {
    let cfg = LlmConfig {
        model: "llama3.1:70b".into(),
        ..LlmConfig::default()
    };
    assert_eq!(cfg.resolved_mode(), AnalysisMode::Advanced);
}
