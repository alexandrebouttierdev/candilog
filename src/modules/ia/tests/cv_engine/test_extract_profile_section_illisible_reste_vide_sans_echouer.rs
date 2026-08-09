//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_extract_profile_section_illisible_reste_vide_sans_echouer() {
    // Seule l'identité répond ; les autres sections restent vides (jamais inventées),
    // et l'extraction réussit malgré tout.
    let identity = r#"{"first_name":"Ada","last_name":"L","email":""}"#;
    let profile = engine(vec![identity])
        .extract_profile("texte")
        .await
        .unwrap();
    assert_eq!(profile.personal.first_name, "Ada");
    assert!(profile.experiences.is_empty());
    assert!(profile.skills.is_empty());
    assert!(profile.certifications.is_empty());
}
