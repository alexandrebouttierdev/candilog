//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_extract_profile_small_rejette_entreprise_et_competence_inventees() {
    // Le LLM renvoie une entreprise (Google) et une compétence (Kubernetes) absentes du CV.
    let identity = r#"{"first_name":"Ada","last_name":"Lovelace","email":""}"#;
    let history = r#"{"experiences":[{"title":"Ing","company":"ACME Corporation"},{"title":"Dev","company":"Google"}],"education":[]}"#;
    let skills = r#"{"skills":[{"name":"Rust"},{"name":"Kubernetes"}],"languages":[]}"#;
    let portfolio = r#"{"projects":[],"certifications":[]}"#;
    let profile = keyed_engine(AnalysisMode::Small, identity, history, skills, portfolio)
        .extract_profile(CV_SOURCE)
        .await
        .unwrap();
    // Grounding actif : seules les valeurs présentes dans le source subsistent.
    assert_eq!(profile.experiences.len(), 1);
    assert_eq!(profile.experiences[0].company, "ACME Corporation");
    assert_eq!(profile.skills.len(), 1);
    assert_eq!(profile.skills[0].name, "Rust");
}
