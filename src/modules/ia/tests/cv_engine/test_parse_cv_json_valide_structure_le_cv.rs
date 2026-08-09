//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_parse_cv_json_valide_structure_le_cv() {
    let body = r#"{"summary":"Dev Rust","experiences":[{"title":"Dev","company":"ACME","description":"Backend"}],"skills":["Rust"],"education":[{"degree":"M2","school":"X"}]}"#;
    let cv = engine(vec![body])
        .parse_cv("texte brut du cv")
        .await
        .unwrap();
    assert_eq!(cv.summary, "Dev Rust");
    assert_eq!(cv.skills, vec!["Rust"]);
    assert_eq!(cv.experiences.len(), 1);
}
