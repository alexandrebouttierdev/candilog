//! Cas de test isolé.

use super::*;

#[test]
fn test_extraction_tolere_scalaires_et_champs_absents() {
    // `phone` en nombre, `skills` en objet isolé, `experiences` absent.
    let profile = parse(
        r#"{"personal":{"first_name":"Ada","last_name":"Lovelace","email":"ada@x.io","phone":33612345678},
                "skills":{"name":"Rust"}}"#,
    );
    assert_eq!(profile.personal.first_name, "Ada");
    assert_eq!(profile.personal.phone.as_deref(), Some("33612345678"));
    assert_eq!(profile.skills.len(), 1);
    assert_eq!(profile.skills[0].name, "Rust");
    assert!(profile.experiences.is_empty());
}
