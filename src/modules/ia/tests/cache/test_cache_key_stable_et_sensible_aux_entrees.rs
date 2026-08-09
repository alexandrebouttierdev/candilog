//! Cas de test isolé.

use super::*;

#[test]
fn test_cache_key_stable_et_sensible_aux_entrees() {
    let a = cache_key("ollama", "gemma3:1b", "small", "parse_cv", "texte");
    let b = cache_key("ollama", "gemma3:1b", "small", "parse_cv", "texte");
    let c = cache_key("ollama", "gemma3:1b", "standard", "parse_cv", "texte");
    assert_eq!(a, b); // déterministe
    assert_ne!(a, c); // le mode change la clé
    assert_eq!(a.len(), 64); // sha256 hex
}
