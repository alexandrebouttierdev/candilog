//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_parse_cv_reessaie_apres_json_invalide() {
    let good = r#"{"summary":"Dev","experiences":[],"skills":[],"education":[]}"#;
    let cv = engine(vec!["pas du json", good])
        .parse_cv("texte")
        .await
        .unwrap();
    assert_eq!(cv.summary, "Dev");
}
