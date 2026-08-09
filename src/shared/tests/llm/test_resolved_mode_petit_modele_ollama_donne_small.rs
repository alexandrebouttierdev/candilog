//! Cas de test isolé.

use super::*;

#[test]
fn test_resolved_mode_petit_modele_ollama_donne_small() {
    let cfg = LlmConfig {
        model: "gemma3:1b".into(),
        ..LlmConfig::default()
    };
    assert_eq!(cfg.resolved_mode(), AnalysisMode::Small);
}
