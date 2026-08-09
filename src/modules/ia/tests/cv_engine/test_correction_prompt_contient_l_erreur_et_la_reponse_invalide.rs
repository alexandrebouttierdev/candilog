//! Cas de test isolé.

use super::*;

#[test]
fn test_correction_prompt_contient_l_erreur_et_la_reponse_invalide() {
    let error =
        serde_json::from_str::<serde_json::Value>(r#"{"skills":["Go" "Python"]}"#).unwrap_err();
    let prompt = correction_prompt("offre Astek", r#"{"skills":["Go" "Python"]}"#, &error);

    assert!(prompt.contains("offre Astek"));
    assert!(prompt.contains("expected `,` or `]`"));
    assert!(prompt.contains(r#"{"skills":["Go" "Python"]}"#));
    assert!(prompt.contains("Output only the JSON object"));
}
