//! Cas de test isolé.

use super::*;

#[test]
fn test_default_utilise_ollama_et_francais() {
    let s = AppSettings::default();
    assert!(matches!(s.llm.provider, ProviderKind::Ollama));
    assert_eq!(s.langue, "fr");
    assert!(matches!(s.theme, ThemePref::System));
}
