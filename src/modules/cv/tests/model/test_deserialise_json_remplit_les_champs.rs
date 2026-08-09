//! Cas de test isolé.

use super::*;

#[test]
fn test_deserialise_json_remplit_les_champs() {
    let json = r#"{"id":"00000000-0000-0000-0000-000000000000","name":"CV Dev","content":{"cv":{"summary":"x"}},"created_at":"2026-07-03T10:00:00Z"}"#;
    let v: CvVersion = serde_json::from_str(json).unwrap();
    assert_eq!(v.name, "CV Dev");
    assert_eq!(v.created_at, "2026-07-03T10:00:00Z");
    assert_eq!(v.content["cv"]["summary"], "x");
}
