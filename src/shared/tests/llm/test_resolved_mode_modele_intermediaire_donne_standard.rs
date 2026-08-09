//! Cas de test isolé.

use super::*;

#[test]
fn test_resolved_mode_modele_intermediaire_donne_standard() {
    let cfg = LlmConfig {
        model: "llama3.2:3b".into(),
        ..LlmConfig::default()
    };
    assert_eq!(cfg.resolved_mode(), AnalysisMode::Standard);
}
