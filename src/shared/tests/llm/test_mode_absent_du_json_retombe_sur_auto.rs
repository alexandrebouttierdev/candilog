//! Cas de test isolé.

use super::*;

#[test]
fn test_mode_absent_du_json_retombe_sur_auto() {
    // Config persistée avant l'ajout du champ `mode` : désérialisation rétro-compatible.
    let json = r#"{"provider":"ollama","api_key":null,"endpoint":"http://x","model":"gemma3:1b","temperature":0.7}"#;
    let cfg: LlmConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.mode, AnalysisMode::Auto);
}
