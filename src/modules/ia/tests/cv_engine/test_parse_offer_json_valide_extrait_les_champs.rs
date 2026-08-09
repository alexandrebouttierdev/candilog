//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_parse_offer_json_valide_extrait_les_champs() {
    let body = r#"{"title":"Dev Rust","skills":["Rust"],"soft_skills":[],"experience":"3 ans","keywords":["async"]}"#;
    let parsed = engine(vec![body]).parse_offer("offre").await.unwrap();
    assert_eq!(parsed.title, "Dev Rust");
    assert_eq!(parsed.skills, vec!["Rust"]);
    assert_eq!(parsed.experience.as_deref(), Some("3 ans"));
}
