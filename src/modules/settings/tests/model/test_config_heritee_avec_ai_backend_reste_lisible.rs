//! Vérifie la lecture d'une configuration héritée.

use super::*;

#[test]
fn test_config_heritee_avec_ai_backend_reste_lisible() {
    let json = r#"{"llm":{"provider":"ollama","api_key":null,"endpoint":"http://localhost:11434",
        "model":"llama3.2:3b","temperature":0.7},"ai_backend":"candilog","theme":"system","langue":"fr"}"#;
    let settings: AppSettings = serde_json::from_str(json).unwrap();
    assert_eq!(settings.langue, "fr");
    assert!(matches!(settings.theme, ThemePref::System));
}
