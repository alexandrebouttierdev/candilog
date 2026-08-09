//! Cas de test isolé.

use super::*;

#[test]
fn test_est_configure_modele_vide_est_faux() {
    let cfg = LlmConfig {
        model: String::new(),
        ..LlmConfig::default()
    };
    assert!(!cfg.est_configure());
}
