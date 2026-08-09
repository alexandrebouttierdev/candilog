//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_extract_profile_assemble_les_4_appels_specialises() {
    // 4 appels séquentiels : identité, parcours, compétences/langues, projets/certifs.
    let identity =
        r#"{"first_name":"Ada","last_name":"Lovelace","email":"ada@x.io","linkedin":"in/ada"}"#;
    let history = r#"{"experiences":[{"title":"Ingénieure","company":"ACME","start_date":"06/2022","current":true}],"education":[]}"#;
    let skills = r#"{"skills":[{"name":"Rust"},{"name":"rust"}],"languages":[{"name":"Anglais","level":"C1"}]}"#;
    let portfolio = r#"{"projects":[],"certifications":[{"name":"AWS","issuer":"Amazon"}]}"#;
    // Le source contient les valeurs attendues : la validation d'ancrage (active en mode
    // Standard) les conserve toutes.
    let source = "Ada Lovelace — ada@x.io\nIngénieure chez ACME (2022).\nCompétences : Rust.\nAnglais C1.\nCertification AWS.";
    let profile = engine(vec![identity, history, skills, portfolio])
        .extract_profile(source)
        .await
        .unwrap();
    assert_eq!(profile.personal.first_name, "Ada");
    assert_eq!(profile.personal.linkedin.as_deref(), Some("in/ada"));
    assert_eq!(profile.experiences.len(), 1);
    assert_eq!(profile.experiences[0].start_date, "2022-06");
    assert!(profile.experiences[0].current);
    assert_eq!(profile.skills.len(), 1); // dédoublonné
    assert_eq!(profile.languages[0].level, "Courant"); // normalisé
    assert_eq!(profile.certifications[0].name, "AWS");
}
