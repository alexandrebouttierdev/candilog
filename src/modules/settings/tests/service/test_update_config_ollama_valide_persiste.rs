//! Cas de test isolé.

use super::*;

#[test]
fn test_update_config_ollama_valide_persiste() {
    let s = AppSettings::default(); // Ollama, pas de clé requise
    let r = service().update(&s).unwrap();
    assert_eq!(r.langue, s.langue);
}
