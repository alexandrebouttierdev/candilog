//! Cas de test isolé.

use super::*;

#[test]
fn test_model_disables_thinking_detecte_les_modeles_a_raisonnement() {
    assert!(model_disables_thinking("gpt-oss:20b"));
    assert!(model_disables_thinking("deepseek-r1:7b"));
    assert!(model_disables_thinking("qwen3:8b"));
    assert!(!model_disables_thinking("llama3.2:3b"));
    assert!(!model_disables_thinking("gemma3:1b"));
    assert!(!model_disables_thinking("qwen2.5:7b"));
}
