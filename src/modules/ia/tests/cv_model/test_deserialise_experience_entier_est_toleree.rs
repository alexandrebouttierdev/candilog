//! Cas de test isolé.

use super::*;

#[test]
fn test_deserialise_experience_entier_est_toleree() {
    let json = r#"{"title":"Dev","skills":["Rust"],"soft_skills":[],"experience":3,"keywords":[]}"#;
    let offer: ParsedOffer = serde_json::from_str(json).unwrap();
    assert_eq!(offer.experience.as_deref(), Some("3"));
}
