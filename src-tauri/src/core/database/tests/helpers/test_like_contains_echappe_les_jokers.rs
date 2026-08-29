//! Cas de test isolé.

use super::*;

#[test]
fn test_like_contains_echappe_les_jokers() {
    assert_eq!(like_contains(""), "%%");
    assert_eq!(like_contains("Nova"), "%nova%");
    assert_eq!(like_contains("%_\\"), "%\\%\\_\\\\%");
}
