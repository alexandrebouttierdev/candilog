//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_extract_profile_advanced_conserve_les_valeurs_non_ancrees() {
    // Mêmes réponses, mode avancé : aucune validation d'ancrage, tout est conservé.
    let identity = r#"{"first_name":"Ada","last_name":"Lovelace","email":""}"#;
    let history = r#"{"experiences":[{"title":"Ing","company":"ACME Corporation"},{"title":"Dev","company":"Google"}],"education":[]}"#;
    let skills = r#"{"skills":[{"name":"Rust"},{"name":"Kubernetes"}],"languages":[]}"#;
    let portfolio = r#"{"projects":[],"certifications":[]}"#;
    let profile = keyed_engine(AnalysisMode::Advanced, identity, history, skills, portfolio)
        .extract_profile(CV_SOURCE)
        .await
        .unwrap();
    assert_eq!(profile.experiences.len(), 2);
    assert_eq!(profile.skills.len(), 2);
}
