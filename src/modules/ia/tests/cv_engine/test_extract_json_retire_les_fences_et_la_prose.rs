//! Cas de test isolé.

use super::*;

#[test]
fn test_extract_json_retire_les_fences_et_la_prose() {
    assert_eq!(
        extract_json("Voici : ```json\n{\"a\":1}\n``` fin"),
        "{\"a\":1}"
    );
    assert_eq!(extract_json("{\"a\":1}"), "{\"a\":1}");
}
