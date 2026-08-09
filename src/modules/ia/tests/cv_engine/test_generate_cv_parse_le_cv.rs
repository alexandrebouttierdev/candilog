//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_generate_cv_parse_le_cv() {
    let body = r#"{"summary":"Dev Rust orienté async","experiences":[{"title":"Dev","company":"ACME","description":"Backend Rust"}],"skills":["Rust"],"education":[{"degree":"M2","school":"X"}]}"#;
    let cv = engine(vec![body])
        .generate_cv(
            &crate::shared::profile::Profile::default(),
            &ParsedOffer::default(),
            &crate::modules::ia::cv_model::MatchScore::default(),
        )
        .await
        .unwrap();
    assert_eq!(cv.summary, "Dev Rust orienté async");
    assert_eq!(cv.experiences.len(), 1);
    assert_eq!(cv.skills, vec!["Rust"]);
}
