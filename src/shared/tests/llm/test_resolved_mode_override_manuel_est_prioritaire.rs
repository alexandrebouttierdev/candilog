//! Cas de test isolé.

use super::*;

#[test]
fn test_resolved_mode_override_manuel_est_prioritaire() {
    // Un 1B forcé en Advanced doit rester Advanced (l'override gagne sur l'heuristique).
    let cfg = LlmConfig {
        model: "gemma3:1b".into(),
        mode: AnalysisMode::Advanced,
        ..LlmConfig::default()
    };
    assert_eq!(cfg.resolved_mode(), AnalysisMode::Advanced);
}
