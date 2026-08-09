//! Cas de test isolé.

use super::*;

#[test]
fn test_deserialise_cv_accepte_les_champs_absents() {
    let cv: GeneratedCv = serde_json::from_str(r#"{"summary":"Dev"}"#).unwrap();
    assert_eq!(cv.summary, "Dev");
    assert!(cv.experiences.is_empty());
    assert!(cv.skills.is_empty());
    assert!(cv.education.is_empty());
}
