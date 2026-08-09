//! Cas de test isolé.

use super::*;

#[test]
fn test_deserialise_cv_accepte_experiences_et_formations_en_chaines() {
    let json = r#"{
            "summary":"Développeur mobile",
            "experiences":["Développé et déployé des applications React Native en production."],
            "skills":["React Native"],
            "education":["Master informatique — Université de Lille"]
        }"#;

    let cv: GeneratedCv = serde_json::from_str(json).unwrap();

    assert_eq!(cv.experiences.len(), 1);
    assert!(cv.experiences[0].description.contains("React Native"));
    assert_eq!(cv.education.len(), 1);
    assert!(cv.education[0].degree.contains("Master informatique"));
}
