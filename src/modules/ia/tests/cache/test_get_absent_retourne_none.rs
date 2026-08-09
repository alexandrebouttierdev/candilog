//! Cas de test isolé.

use super::*;

#[test]
fn test_get_absent_retourne_none() {
    assert_eq!(repo().get("inconnu").unwrap(), None);
}
