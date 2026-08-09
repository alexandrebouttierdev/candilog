//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_extract_profile_reessaie_la_section_apres_json_invalide() {
    // L'identité échoue puis réussit (retry interne) ; les sections suivantes sont vides.
    let good = r#"{"first_name":"Ada","last_name":"L","email":""}"#;
    let profile = engine(vec!["pas du json", good])
        .extract_profile("texte")
        .await
        .unwrap();
    assert_eq!(profile.personal.first_name, "Ada");
}
