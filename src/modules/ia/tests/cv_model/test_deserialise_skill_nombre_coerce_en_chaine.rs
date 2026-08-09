//! Cas de test isolé.

use super::*;

#[test]
fn test_deserialise_skill_nombre_coerce_en_chaine() {
    let json =
        r#"{"title":"Dev","skills":["Rust",3],"soft_skills":[],"experience":null,"keywords":[]}"#;
    let offer: ParsedOffer = serde_json::from_str(json).unwrap();
    assert_eq!(offer.skills, vec!["Rust", "3"]);
}
