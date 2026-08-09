//! Cas de test isolé.

use super::*;

#[test]
fn test_sse_data_extrait_la_charge_et_ignore_le_reste() {
    assert_eq!(sse_data("data: {\"a\":1}"), Some("{\"a\":1}"));
    assert_eq!(sse_data("data:{\"a\":1}"), Some("{\"a\":1}"));
    assert_eq!(sse_data("event: delta"), None);
    assert_eq!(sse_data(""), None);
    assert_eq!(sse_data("data: [DONE]"), None);
}
