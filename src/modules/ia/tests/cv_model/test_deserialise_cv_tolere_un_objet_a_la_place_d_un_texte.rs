//! Cas de test isolé.

use super::*;

#[test]
fn test_deserialise_cv_tolere_un_objet_a_la_place_d_un_texte() {
    let json = r#"{
            "summary":{"profil":"Développeur Rust","experience":"3 ans"},
            "experiences":[{
                "title":{"poste":"Backend developer"},
                "company":"ACME",
                "description":{"missions":["API", "Tests"]}
            }],
            "skills":["Rust", 3],
            "education":[]
        }"#;

    let cv: GeneratedCv = serde_json::from_str(json).unwrap();

    assert!(cv.summary.contains("Développeur Rust"));
    assert!(cv.summary.contains("3 ans"));
    assert_eq!(cv.experiences[0].title, "Backend developer");
    assert_eq!(cv.experiences[0].description, "API, Tests");
    assert_eq!(cv.skills, vec!["Rust", "3"]);
}
