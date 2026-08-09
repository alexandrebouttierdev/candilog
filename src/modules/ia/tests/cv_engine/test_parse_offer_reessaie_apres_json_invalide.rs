//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_parse_offer_reessaie_apres_json_invalide() {
    let good = r#"{"title":"Dev","skills":[],"soft_skills":[],"experience":null,"keywords":[]}"#;
    let parsed = engine(vec!["pas du json", good])
        .parse_offer("offre")
        .await
        .unwrap();
    assert_eq!(parsed.title, "Dev");
}
