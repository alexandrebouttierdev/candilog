//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_extract_profile_contacts_regex_priment_sur_le_llm() {
    // Le LLM invente un e-mail : la regex extrait le vrai depuis le CV et l'emporte.
    let identity = r#"{"first_name":"Ada","last_name":"Lovelace","email":"faux@invente.com"}"#;
    let empty = r#"{"experiences":[],"education":[],"skills":[],"languages":[],"projects":[],"certifications":[]}"#;
    let profile = keyed_engine(AnalysisMode::Standard, identity, empty, empty, empty)
        .extract_profile(CV_SOURCE)
        .await
        .unwrap();
    assert_eq!(profile.personal.email, "ada@x.io");
}
