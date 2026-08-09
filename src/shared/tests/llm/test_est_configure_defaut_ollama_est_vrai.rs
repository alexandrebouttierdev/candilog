//! Cas de test isolé.

use super::*;

#[test]
fn test_est_configure_defaut_ollama_est_vrai() {
    assert!(LlmConfig::default().est_configure());
}
